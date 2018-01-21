#![allow(unused)]

extern crate num;

use self::num::NumCast;
use config::Signature;
use memlib::{Bitness, Module, Process};
use std::mem;

pub type Result<T> = ::std::result::Result<T, ScanError>;

#[derive(Debug, Fail)]
pub enum ScanError {
    #[fail(display = "Module not found")] ModuleNotFound,

    #[fail(display = "Pattern not found")] PatternNotFound,

    #[fail(display = "Offset out of module bounds")] OffsetOutOfBounds,

    #[fail(display = "rip_relative failed")] RIPRelativeFailed,
}

pub fn find_signature(sig: &Signature, process: &Process) -> Result<usize> {
    debug!("Begin scan: {}", sig.name);
    debug!("Bitness: {:?}", process.bitness);
    debug!("Load module {}", sig.module);
    let module = process
        .get_module(&sig.module)
        .ok_or(ScanError::ModuleNotFound)?;
    debug!(
        "Module found: {} - Base: 0x{:X} Size: 0x{:X}",
        module.name, module.base, module.size
    );

    debug!("Searching pattern: {}", sig.pattern);
    let mut addr = module
        .find_pattern(&sig.pattern)
        .ok_or(ScanError::PatternNotFound)?;
    debug!("Pattern found at: 0x{:X}", addr);

    for (i, o) in sig.offsets.iter().enumerate() {
        debug!("Offset #{}: ptr: 0x{:X} offset: 0x{:X}", i, addr, o);

        let pos = (addr as isize + o) as usize;
        let data = module.data.get(pos).ok_or_else(|| {
            debug!(
                "WARN OOB - ptr: 0x{:X} module size: 0x{:X}",
                pos, module.size
            );
            ScanError::OffsetOutOfBounds
        })?;

        let tmp = match process.bitness {
            Bitness::X86 => {
                let raw: u32 = unsafe { mem::transmute_copy(data) };
                raw as usize
            }
            Bitness::X64 => {
                let raw: u64 = unsafe { mem::transmute_copy(data) };
                raw as usize
            }
        };

        addr = tmp - module.base;
        debug!("Offset #{}: raw: 0x{:X} - base => 0x{:X}", i, tmp, addr);
    }

    if sig.rip_relative {
        let addr_is_relative = match sig.offsets.len() {
            0 => false,
            _ => true,
        };
        debug!("rip_relative: addr_is_relative {}", addr_is_relative);
        let rip: u32 = module
            .get_raw(addr, addr_is_relative)
            .ok_or(ScanError::RIPRelativeFailed)?;
        debug!("rip_relative: addr 0x{:X} + rip 0x{:X} + 0x4", addr, rip);
        addr += rip as usize + ::std::mem::size_of::<u32>();
        debug!("rip_relative: addr => 0x{:X}", addr);
    }

    addr = (addr as isize + sig.extra) as usize;
    if !sig.relative {
        addr = match sig.offsets.len() {
            0 => addr,
            _ => addr + module.base,
        }
    }

    Ok(addr)
}
