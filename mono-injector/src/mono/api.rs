use std::collections::HashMap;

use crate::asm::StubBuilder;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::pe::Export;
use crate::process::arch::Arch;
use crate::process::memory::{
    RemoteAllocation, RemoteStr, SharedProcess, execute_remote, read_bytes, read_ptr, read_u32,
};

const MONO_GET_ROOT_DOMAIN: &str = "mono_get_root_domain";
const MONO_THREAD_ATTACH: &str = "mono_thread_attach";
const MONO_IMAGE_OPEN_FROM_DATA: &str = "mono_image_open_from_data";
const MONO_ASSEMBLY_LOAD_FROM_FULL: &str = "mono_assembly_load_from_full";
const MONO_ASSEMBLY_GET_IMAGE: &str = "mono_assembly_get_image";
const MONO_CLASS_FROM_NAME: &str = "mono_class_from_name";
const MONO_CLASS_GET_METHOD_FROM_NAME: &str = "mono_class_get_method_from_name";
const MONO_RUNTIME_INVOKE: &str = "mono_runtime_invoke";
const MONO_ASSEMBLY_CLOSE: &str = "mono_assembly_close";
const MONO_IMAGE_STRERROR: &str = "mono_image_strerror";
const MONO_OBJECT_GET_CLASS: &str = "mono_object_get_class";
const MONO_CLASS_GET_NAME: &str = "mono_class_get_name";

const REQUIRED: &[&str] = &[
    MONO_GET_ROOT_DOMAIN,
    MONO_THREAD_ATTACH,
    MONO_IMAGE_OPEN_FROM_DATA,
    MONO_ASSEMBLY_LOAD_FROM_FULL,
    MONO_ASSEMBLY_GET_IMAGE,
    MONO_CLASS_FROM_NAME,
    MONO_CLASS_GET_METHOD_FROM_NAME,
    MONO_RUNTIME_INVOKE,
    MONO_ASSEMBLY_CLOSE,
    MONO_IMAGE_STRERROR,
    MONO_OBJECT_GET_CLASS,
    MONO_CLASS_GET_NAME,
];

/// Resolved addresses of the 12 required Mono exports in the target process.
pub(crate) struct RemoteMonoApi {
    functions: HashMap<&'static str, u64>,
}

impl RemoteMonoApi {
    /// Resolves all 12 required symbols from the export list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ExportNotFound`] for any missing symbol.
    pub(crate) fn resolve(exports: &[Export]) -> Result<Self> {
        let map: HashMap<_, u64> = exports
            .iter()
            .map(|e| (e.name.as_str(), e.address))
            .collect();

        let functions = REQUIRED
            .iter()
            .map(|&name| {
                let addr = map.get(name).copied().ok_or(Error::ExportNotFound(name))?;
                Ok((name, addr))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Self { functions })
    }

    pub(crate) fn lookup(&self, name: &'static str) -> Result<u64> {
        self.functions
            .get(name)
            .copied()
            .ok_or(Error::ExportNotFound(name))
    }
}

/// All state required to execute a sequence of remote Mono API calls.
///
/// Call [`attach`](Self::attach) first — this fetches the root domain and enables the
/// `mono_thread_attach` preamble that the Mono GC requires on every remote thread.
pub(crate) struct MonoSession {
    api: RemoteMonoApi,
    process: SharedProcess,
    timeout_ms: u32,
    base_dir: String,
    root_domain: Option<u64>,
}

impl MonoSession {
    pub(crate) fn new(api: RemoteMonoApi, process: SharedProcess, config: &Config) -> Self {
        Self {
            api,
            process,
            timeout_ms: config.timeout_ms,
            base_dir: config.base_dir.clone(),
            root_domain: None,
        }
    }

    /// Calls `mono_get_root_domain` and caches the result.
    ///
    /// After this succeeds, every subsequent [`call`](Self::call) includes a
    /// `mono_thread_attach` preamble.
    pub(crate) fn attach(&mut self) -> Result<()> {
        let fn_ptr = self.api.lookup(MONO_GET_ROOT_DOMAIN)?;
        let root_domain = self.call(fn_ptr, &[])?;
        if root_domain == 0 {
            return Err(Error::NullRootDomain);
        }
        self.root_domain = Some(root_domain);
        Ok(())
    }

    pub(crate) fn open_image(&self, data: &[u8]) -> Result<u64> {
        let data_alloc = RemoteAllocation::new_with_data(&self.process, data)?;
        let status_alloc = RemoteAllocation::new(&self.process, 4)?;
        let fn_ptr = self.api.lookup(MONO_IMAGE_OPEN_FROM_DATA)?;
        let result = self.call(
            fn_ptr,
            &[
                data_alloc.address(),
                data.len() as u64,
                1,
                status_alloc.address(),
            ],
        )?;
        self.check_image_status(&status_alloc, result)
    }

    pub(crate) fn open_assembly(&self, image: u64) -> Result<u64> {
        let base_dir = RemoteStr::new(&self.process, &self.base_dir)?;
        let status_alloc = RemoteAllocation::new(&self.process, 4)?;
        let fn_ptr = self.api.lookup(MONO_ASSEMBLY_LOAD_FROM_FULL)?;
        let result = self.call(
            fn_ptr,
            &[image, base_dir.address(), status_alloc.address(), 0],
        )?;
        if result == 0 {
            Err(Error::AssemblyLoadFailed)
        } else {
            Ok(result)
        }
    }

    pub(crate) fn get_image(&self, assembly: u64) -> Result<u64> {
        let fn_ptr = self.api.lookup(MONO_ASSEMBLY_GET_IMAGE)?;
        let result = self.call(fn_ptr, &[assembly])?;
        if result == 0 {
            Err(Error::NullImage)
        } else {
            Ok(result)
        }
    }

    pub(crate) fn get_class(&self, image: u64, namespace: &str, class_name: &str) -> Result<u64> {
        let ns = RemoteStr::new(&self.process, namespace)?;
        let cls = RemoteStr::new(&self.process, class_name)?;
        let fn_ptr = self.api.lookup(MONO_CLASS_FROM_NAME)?;
        let result = self.call(fn_ptr, &[image, ns.address(), cls.address()])?;
        if result == 0 {
            return Err(Error::ClassNotFound {
                namespace: namespace.to_owned(),
                name: class_name.to_owned(),
            });
        }
        Ok(result)
    }

    pub(crate) fn get_method(&self, class: u64, method_name: &str) -> Result<u64> {
        let name = RemoteStr::new(&self.process, method_name)?;
        let fn_ptr = self.api.lookup(MONO_CLASS_GET_METHOD_FROM_NAME)?;
        let result = self.call(fn_ptr, &[class, name.address(), 0])?;
        if result == 0 {
            Err(Error::MethodNotFound(method_name.to_owned()))
        } else {
            Ok(result)
        }
    }

    /// Invokes `method` and checks for a thrown managed exception.
    pub(crate) fn invoke(&self, method: u64) -> Result<()> {
        let exc_alloc = RemoteAllocation::new(&self.process, self.process.arch.ptr_size())?;
        let fn_ptr = self.api.lookup(MONO_RUNTIME_INVOKE)?;
        self.call(fn_ptr, &[method, 0, 0, exc_alloc.address()])?;
        self.check_exception(&exc_alloc)
    }

    pub(crate) fn close_assembly(&self, assembly: u64) -> Result<()> {
        let fn_ptr = self.api.lookup(MONO_ASSEMBLY_CLOSE)?;
        self.call(fn_ptr, &[assembly])?;
        Ok(())
    }

    fn call(&self, fn_ptr: u64, args: &[u64]) -> Result<u64> {
        let ret_alloc = RemoteAllocation::new(&self.process, self.process.arch.ptr_size())?;
        let attach = self.thread_attach_params()?;
        let builder = args.iter().fold(
            StubBuilder::new(self.process.arch, fn_ptr, ret_alloc.address()),
            |b, &a| b.arg(a),
        );
        let builder = match attach {
            Some((ap, rd)) => builder.with_thread_attach(ap, rd),
            None => builder,
        };
        let stub = builder.build()?;
        execute_remote(
            &self.process,
            stub.bytes(),
            ret_alloc.address(),
            self.timeout_ms,
        )
    }

    fn thread_attach_params(&self) -> Result<Option<(u64, u64)>> {
        match self.root_domain {
            None => Ok(None),
            Some(root) => Ok(Some((self.api.lookup(MONO_THREAD_ATTACH)?, root))),
        }
    }

    fn check_image_status(&self, status_alloc: &RemoteAllocation, result: u64) -> Result<u64> {
        let mut buf = [0u8; 4];
        status_alloc.read_bytes(&mut buf)?;
        let status = u32::from_le_bytes(buf);
        if status == 0 {
            return Ok(result);
        }
        let message = self.read_strerror(status).unwrap_or_default();
        Err(Error::ImageOpenFailed {
            status: status_to_enum(status),
            message,
        })
    }

    fn read_strerror(&self, status: u32) -> Result<String> {
        let fn_ptr = self.api.lookup(MONO_IMAGE_STRERROR)?;
        let str_ptr = self.call(fn_ptr, &[u64::from(status)])?;
        if str_ptr == 0 {
            return Ok(String::new());
        }
        read_cstring(&self.process, str_ptr)
    }

    fn check_exception(&self, exc_alloc: &RemoteAllocation) -> Result<()> {
        let exc = read_ptr(&self.process, exc_alloc.address())?;
        if exc == 0 {
            return Ok(());
        }
        let class_name = self
            .read_exception_class(exc)
            .unwrap_or_else(|_| "<unknown>".to_owned());
        let message = self.read_exception_message(exc).unwrap_or_default();
        Err(Error::ManagedException {
            class_name,
            message,
        })
    }

    fn read_exception_class(&self, exc: u64) -> Result<String> {
        let get_class_fn = self.api.lookup(MONO_OBJECT_GET_CLASS)?;
        let class_ptr = self.call(get_class_fn, &[exc])?;
        if class_ptr == 0 {
            return Ok("<unknown>".to_owned());
        }
        let get_name_fn = self.api.lookup(MONO_CLASS_GET_NAME)?;
        let name_ptr = self.call(get_name_fn, &[class_ptr])?;
        if name_ptr == 0 {
            Ok("<unknown>".to_owned())
        } else {
            read_cstring(&self.process, name_ptr)
        }
    }

    fn read_exception_message(&self, exc: u64) -> Result<String> {
        let (str_offset, len_offset, chars_offset) = match self.process.arch {
            Arch::X64 => (0x20u64, 0x10u64, 0x14u64),
            Arch::X86 => (0x10u64, 0x08u64, 0x0Cu64),
        };
        let str_ptr = read_ptr(&self.process, exc + str_offset)?;
        if str_ptr == 0 {
            return Ok(String::new());
        }
        read_mono_string(&self.process, str_ptr, len_offset, chars_offset)
    }
}

fn read_mono_string(
    process: &crate::process::memory::ProcessHandle,
    str_ptr: u64,
    len_offset: u64,
    chars_offset: u64,
) -> Result<String> {
    let length = read_u32(process, str_ptr + len_offset)? as usize;
    if length == 0 {
        return Ok(String::new());
    }
    let mut buf = vec![0u8; length * 2];
    read_bytes(process, str_ptr + chars_offset, &mut buf)?;
    let chars: Vec<u16> = buf
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Ok(String::from_utf16_lossy(&chars))
}

fn read_cstring(process: &crate::process::memory::ProcessHandle, addr: u64) -> Result<String> {
    let mut buf = [0u8; 256];
    read_bytes(process, addr, &mut buf)?;
    let null = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Ok(String::from_utf8_lossy(&buf[..null]).into_owned())
}

fn status_to_enum(status: u32) -> mono_rt::MonoImageOpenStatus {
    match status {
        1 => mono_rt::MonoImageOpenStatus::ErrorErrno,
        2 => mono_rt::MonoImageOpenStatus::MissingAssemblyRef,
        _ => mono_rt::MonoImageOpenStatus::ImageInvalid,
    }
}
