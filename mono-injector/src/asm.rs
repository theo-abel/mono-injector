use iced_x86::code_asm::{
    CodeAssembler, dword_ptr, eax, esp, qword_ptr, r8, r9, r11, rax, rcx, rdx, rsp,
};

use crate::error::{Error, Result};
use crate::process::arch::Arch;

/// Assembled shellcode bytes ready to be written into a remote process.
pub(crate) struct Stub(Vec<u8>);

impl Stub {
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Builds a shellcode stub for a single remote Mono API call.
///
/// Construct with [`new`](Self::new), optionally chain [`arg`](Self::arg) and
/// [`with_thread_attach`](Self::with_thread_attach), then call [`build`](Self::build).
pub(crate) struct StubBuilder {
    arch: Arch,
    fn_ptr: u64,
    ret_val_ptr: u64,
    args: Vec<u64>,
    thread_attach: Option<(u64, u64)>, // (attach_fn_ptr, root_domain)
}

impl StubBuilder {
    pub(crate) fn new(arch: Arch, fn_ptr: u64, ret_val_ptr: u64) -> Self {
        Self {
            arch,
            fn_ptr,
            ret_val_ptr,
            args: Vec::new(),
            thread_attach: None,
        }
    }

    /// Appends one argument (in-order; ABI ordering is handled internally).
    #[must_use]
    pub(crate) fn arg(mut self, val: u64) -> Self {
        self.args.push(val);
        self
    }

    /// Emits a `mono_thread_attach(root_domain)` preamble before the main call.
    #[must_use]
    pub(crate) fn with_thread_attach(mut self, fn_ptr: u64, root_domain: u64) -> Self {
        self.thread_attach = Some((fn_ptr, root_domain));
        self
    }

    /// Assembles the stub using iced-x86.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Assemble`] if iced-x86 fails to encode the instructions.
    pub(crate) fn build(self) -> Result<Stub> {
        match self.arch {
            Arch::X86 => build_x86(&self),
            Arch::X64 => build_x64(&self),
        }
    }
}

/// Validates that `val` fits in 32 bits (required for x86 address/pointer encoding).
fn to_u32(val: u64) -> Result<u32> {
    u32::try_from(val)
        .map_err(|_| Error::Assemble(format!("value {val:#x} exceeds 32-bit address range")))
}

/// Reinterprets a u32 bit pattern as i32 without triggering `cast_possible_wrap`.
fn cast_i32(v: u32) -> i32 {
    i32::from_ne_bytes(v.to_ne_bytes())
}

fn build_x86(b: &StubBuilder) -> Result<Stub> {
    let mut a = CodeAssembler::new(32)?;
    if let Some((attach_ptr, root)) = b.thread_attach {
        a.push(cast_i32(to_u32(root)?))?;
        a.mov(eax, cast_i32(to_u32(attach_ptr)?))?;
        a.call(eax)?;
        a.add(esp, 4i32)?;
    }
    for &arg in b.args.iter().rev() {
        a.push(cast_i32(to_u32(arg)?))?;
    }
    a.mov(eax, cast_i32(to_u32(b.fn_ptr)?))?;
    a.call(eax)?;
    if !b.args.is_empty() {
        let n = i32::try_from(b.args.len())
            .map_err(|_| Error::Assemble("argument count overflow".into()))?
            * 4;
        a.add(esp, n)?;
    }
    a.mov(dword_ptr(u64::from(to_u32(b.ret_val_ptr)?)), eax)?;
    a.ret()?;
    Ok(Stub(a.assemble(0)?))
}

fn build_x64(b: &StubBuilder) -> Result<Stub> {
    let mut a = CodeAssembler::new(64)?;
    a.sub(rsp, 40i32)?;
    if let Some((attach_ptr, root)) = b.thread_attach {
        a.mov(rax, attach_ptr)?;
        a.mov(rcx, root)?;
        a.call(rax)?;
    }
    a.mov(rax, b.fn_ptr)?;
    emit_x64_args(&mut a, &b.args)?;
    a.call(rax)?;
    a.add(rsp, 40i32)?;
    a.mov(r11, b.ret_val_ptr)?;
    a.mov(qword_ptr(r11), rax)?;
    a.ret()?;
    Ok(Stub(a.assemble(0)?))
}

fn emit_x64_args(a: &mut CodeAssembler, args: &[u64]) -> Result<()> {
    let regs = [rcx, rdx, r8, r9];
    for (&arg, &reg) in args.iter().zip(regs.iter()) {
        a.mov(reg, arg)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x64_stub_contains_ret() {
        let stub = StubBuilder::new(Arch::X64, 0x1234_5678_9abc_def0, 0xdead_beef_0000_0000)
            .build()
            .expect("assembly must succeed");
        assert!(!stub.bytes().is_empty());
        assert_eq!(*stub.bytes().last().unwrap(), 0xC3);
    }

    #[test]
    fn x86_stub_contains_ret() {
        let stub = StubBuilder::new(Arch::X86, 0x1234_5678, 0xdead_beef)
            .build()
            .expect("assembly must succeed");
        assert!(!stub.bytes().is_empty());
        assert_eq!(*stub.bytes().last().unwrap(), 0xC3);
    }

    #[test]
    fn x64_with_args_and_attach() {
        let stub = StubBuilder::new(Arch::X64, 0x1111_2222_3333_4444, 0x5555_6666_7777_8888)
            .arg(0xaaaa_bbbb_cccc_dddd)
            .arg(0x1122_3344_5566_7788)
            .with_thread_attach(0x9999_aaaa_bbbb_cccc, 0xffff_eeee_dddd_cccc)
            .build()
            .expect("assembly with attach must succeed");
        assert!(!stub.bytes().is_empty());
    }

    #[test]
    fn x86_with_args_and_attach() {
        let stub = StubBuilder::new(Arch::X86, 0x1234_5678, 0xdead_beef)
            .arg(0x1111_2222)
            .arg(0x3333_4444)
            .with_thread_attach(0xaaaa_bbbb, 0xcccc_dddd)
            .build()
            .expect("assembly with attach must succeed");
        assert!(!stub.bytes().is_empty());
    }
}
