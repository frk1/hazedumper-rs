#![allow(unused)]

extern crate num;

use self::num::NumCast;
use config::Signature;
use memlib::{Module, Process};
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
    debug!("IsWow64: {:?}", process.is_wow64);
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
    debug!(
        "Pattern found at: 0x{:X} (+ base = 0x{:X})",
        addr,
        addr + module.base
    );

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

        let tmp = match process.is_wow64 {
            true => {
                let raw: u32 = unsafe { mem::transmute_copy(data) };
                raw as usize
            }
            false => {
                let raw: u64 = unsafe { mem::transmute_copy(data) };
                raw as usize
            }
        };

        addr = tmp - module.base;
        debug!("Offset #{}: raw: 0x{:X} - base => 0x{:X}", i, tmp, addr);
    }

    if sig.rip_relative {
        debug!(
            "rip_relative: addr 0x{:X} + rip_offset 0x{:X}",
            addr, sig.rip_offset
        );
        addr = (addr as isize + sig.rip_offset) as usize;
        debug!("rip_relative: addr = 0x{:X}", addr);

        let rip: u32 = module
            .get_raw(addr, true)
            .ok_or(ScanError::RIPRelativeFailed)?;

        debug!(
            "rip_relative: addr 0x{:X} + rip 0x{:X} + 0x{:X}",
            addr,
            rip,
            ::std::mem::size_of::<u32>()
        );
        addr += rip as usize + ::std::mem::size_of::<u32>();
        debug!("rip_relative: addr => 0x{:X}", addr);
    }

    debug!("Adding extra 0x{:X}", sig.extra);
    addr = (addr as isize + sig.extra) as usize;
    if !sig.relative {
        debug!(
            "Not relative, addr 0x{:X} + base 0x{:X} => 0x{:X}",
            addr,
            module.base,
            addr + module.base
        );
        addr += module.base;
    }

    Ok(addr)
}
