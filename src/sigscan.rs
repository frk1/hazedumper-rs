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
}

pub fn find_signature32(sig: &Signature, process: &Process) -> Result<usize> {
    debug!("Begin scan: {}", sig.name);

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

        let raw: u32 = unsafe { mem::transmute_copy(data) };

        debug!(
            "Offset #{}: raw: 0x{:X} - base => 0x{:X}",
            i,
            raw,
            (raw as usize - module.base)
        );

        addr = raw as usize - module.base;
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
