use std::sync::Arc;

use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_EVENT};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE,
    PAGE_GUARD, PAGE_NOACCESS, VirtualAllocEx, VirtualFreeEx, VirtualQueryEx,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, LPTHREAD_START_ROUTINE, OpenProcess, PROCESS_CREATE_THREAD,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    WaitForSingleObject,
};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::process::arch::{Arch, detect};

/// RAII wrapper around a process `HANDLE`; closes the handle on drop.
#[derive(Debug)]
pub(crate) struct ProcessHandle {
    pub(crate) raw: HANDLE,
    pub(crate) arch: Arch,
    pub(crate) pid: u32,
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.raw);
        }
    }
}

// Windows process handles are kernel objects and can be used safely from any thread.
unsafe impl Send for ProcessHandle {}
unsafe impl Sync for ProcessHandle {}

pub(crate) type SharedProcess = Arc<ProcessHandle>;

/// Opens the target process with the minimum rights required for injection.
pub(crate) fn open(pid: u32, _config: &Config) -> Result<SharedProcess> {
    let access = PROCESS_CREATE_THREAD
        | PROCESS_VM_OPERATION
        | PROCESS_VM_READ
        | PROCESS_VM_WRITE
        | PROCESS_QUERY_INFORMATION;

    let raw = unsafe { OpenProcess(access, false, pid) }
        .map_err(|e| Error::OpenProcess { pid, source: e })?;

    let arch = detect(raw)?;
    Ok(Arc::new(ProcessHandle { raw, arch, pid }))
}

/// A region of memory allocated in the target process; freed with `VirtualFreeEx` on drop.
#[derive(Debug)]
pub(crate) struct RemoteAllocation {
    address: u64,
    process: SharedProcess,
}

impl RemoteAllocation {
    /// Allocates `size` bytes in the target process with `PAGE_EXECUTE_READWRITE` protection.
    pub(crate) fn new(process: &SharedProcess, size: usize) -> Result<Self> {
        let ptr = unsafe {
            VirtualAllocEx(
                process.raw,
                None,
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };
        if ptr.is_null() {
            return Err(Error::VirtualAlloc(windows::core::Error::from_thread()));
        }
        Ok(Self {
            address: ptr as u64,
            process: Arc::clone(process),
        })
    }

    /// Allocates and immediately writes `data` into the target process.
    pub(crate) fn new_with_data(process: &SharedProcess, data: &[u8]) -> Result<Self> {
        let alloc = Self::new(process, data.len().max(1))?;
        alloc.write(data)?;
        Ok(alloc)
    }

    pub(crate) fn address(&self) -> u64 {
        self.address
    }

    pub(crate) fn write(&self, data: &[u8]) -> Result<()> {
        write_bytes(&self.process, self.address, data)
    }

    pub(crate) fn read_bytes(&self, buf: &mut [u8]) -> Result<()> {
        read_bytes(&self.process, self.address, buf)
    }
}

impl Drop for RemoteAllocation {
    fn drop(&mut self) {
        if let Err(e) =
            unsafe { VirtualFreeEx(self.process.raw, self.address as *mut _, 0, MEM_RELEASE) }
        {
            tracing::warn!("VirtualFreeEx failed for {:#x}: {e}", self.address);
        }
    }
}

/// A null-terminated UTF-8 string allocated in the target process; freed on drop.
pub(crate) struct RemoteStr(RemoteAllocation);

impl RemoteStr {
    pub(crate) fn new(process: &SharedProcess, s: &str) -> Result<Self> {
        let bytes: Vec<u8> = s.bytes().chain(std::iter::once(0u8)).collect();
        Ok(Self(RemoteAllocation::new_with_data(process, &bytes)?))
    }

    pub(crate) fn address(&self) -> u64 {
        self.0.address()
    }
}

pub(crate) fn read_bytes(process: &ProcessHandle, addr: u64, buf: &mut [u8]) -> Result<()> {
    unsafe {
        ReadProcessMemory(
            process.raw,
            addr as *const _,
            buf.as_mut_ptr().cast(),
            buf.len(),
            None,
        )
    }
    .map_err(|e| Error::ReadMemory { addr, source: e })
}

pub(crate) fn write_bytes(process: &ProcessHandle, addr: u64, data: &[u8]) -> Result<()> {
    unsafe {
        WriteProcessMemory(
            process.raw,
            addr as *const _,
            data.as_ptr().cast(),
            data.len(),
            None,
        )
    }
    .map_err(|e| Error::WriteMemory { addr, source: e })
}

pub(crate) fn read_u32(process: &ProcessHandle, addr: u64) -> Result<u32> {
    let mut buf = [0u8; 4];
    read_bytes(process, addr, &mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub(crate) fn read_u64(process: &ProcessHandle, addr: u64) -> Result<u64> {
    let mut buf = [0u8; 8];
    read_bytes(process, addr, &mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub(crate) fn read_ptr(process: &ProcessHandle, addr: u64) -> Result<u64> {
    match process.arch {
        Arch::X86 => read_u32(process, addr).map(u64::from),
        Arch::X64 => read_u64(process, addr),
    }
}

pub(crate) fn is_readable_ptr(process: &ProcessHandle, addr: u64) -> bool {
    query_region(process, addr).is_some_and(|info| region_contains_ptr(process, &info, addr))
}

fn query_region(process: &ProcessHandle, addr: u64) -> Option<MEMORY_BASIC_INFORMATION> {
    let mut info = unsafe { std::mem::zeroed::<MEMORY_BASIC_INFORMATION>() };
    let len = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    let read = unsafe { VirtualQueryEx(process.raw, Some(addr as *const _), &raw mut info, len) };
    (read != 0).then_some(info)
}

fn region_contains_ptr(
    process: &ProcessHandle,
    info: &MEMORY_BASIC_INFORMATION,
    addr: u64,
) -> bool {
    let Some(end) = addr.checked_add(process.arch.ptr_size() as u64) else {
        return false;
    };
    region_is_readable(info) && address_range_contains(info, addr, end)
}

fn region_is_readable(info: &MEMORY_BASIC_INFORMATION) -> bool {
    info.State == MEM_COMMIT && info.Protect.0 & (PAGE_NOACCESS.0 | PAGE_GUARD.0) == 0
}

fn address_range_contains(info: &MEMORY_BASIC_INFORMATION, start: u64, end: u64) -> bool {
    let base = info.BaseAddress as u64;
    let Some(region_end) = base.checked_add(info.RegionSize as u64) else {
        return false;
    };
    start >= base && end <= region_end
}

/// RAII handle to a remote thread; closes the underlying `HANDLE` on drop.
struct RemoteThread(HANDLE);

impl Drop for RemoteThread {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn spawn_remote_thread(process: &ProcessHandle, code_addr: u64) -> Result<RemoteThread> {
    let start: LPTHREAD_START_ROUTINE = Some(unsafe {
        std::mem::transmute::<u64, unsafe extern "system" fn(*mut std::ffi::c_void) -> u32>(
            code_addr,
        )
    });

    let handle = unsafe { CreateRemoteThread(process.raw, None, 0, start, None, 0, None) }
        .map_err(Error::CreateRemoteThread)?;

    Ok(RemoteThread(handle))
}

fn wait_for_thread(thread: &RemoteThread, timeout_ms: u32) -> Result<()> {
    const WAIT_OBJECT_0: WAIT_EVENT = WAIT_EVENT(0);
    const WAIT_TIMEOUT: WAIT_EVENT = WAIT_EVENT(0x102);

    let result = unsafe { WaitForSingleObject(thread.0, timeout_ms) };
    if result == WAIT_TIMEOUT {
        return Err(Error::RemoteThreadTimeout(timeout_ms));
    }

    if result != WAIT_OBJECT_0 {
        return Err(Error::WaitFailed(windows::core::Error::from_thread()));
    }

    Ok(())
}

/// Writes shellcode into the target process, runs it via `CreateRemoteThread`, waits for
/// completion, and returns the value written to `ret_val_addr` by the stub.
pub(crate) fn execute_remote(
    process: &SharedProcess,
    code: &[u8],
    ret_val_addr: u64,
    timeout_ms: u32,
) -> Result<u64> {
    let code_alloc = RemoteAllocation::new_with_data(process, code)?;
    let thread = spawn_remote_thread(process, code_alloc.address())?;

    wait_for_thread(&thread, timeout_ms)?;

    match process.arch {
        Arch::X86 => read_u32(process, ret_val_addr).map(u64::from),
        Arch::X64 => read_u64(process, ret_val_addr),
    }
}
