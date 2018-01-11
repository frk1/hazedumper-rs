extern crate num;
extern crate winapi;

use std::{mem, ptr};

use findpattern;
use snapshot::SnapshotHandle;

use self::winapi::shared::basetsd::SIZE_T;
use self::winapi::shared::minwindef::{DWORD, LPCVOID, LPVOID};
use self::winapi::shared::minwindef::FALSE;
use self::winapi::shared::ntdef::HANDLE;
use self::winapi::um::memoryapi::ReadProcessMemory;
use self::winapi::um::tlhelp32::{MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
                                 TH32CS_SNAPMODULE32};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub data: Vec<u8>,
}

/// Enum for the different signature modes:
///
/// - `Nop`: No operation
/// - `Read`: Read address
/// - `Substract`: Subtract base address
/// - `ReadSubtract`: Read and subtract base address
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Nop,
    Read,
    Subtract,
    ReadSubtract,
}

impl Module {
    fn new() -> Self {
        Module {
            name: "".to_string(),
            base: 0,
            size: 0,
            data: vec![],
        }
    }

    pub fn find_pattern<T>(
        &self,
        pattern: &str,
        mode: Mode,
        offset: i32,
        extra: i32,
    ) -> Option<usize>
    where
        T: num::NumCast,
    {
        findpattern::find_pattern(&self.data, pattern).and_then(|pos| {
            let mut pos = pos as isize;
            pos += offset as isize;

            pos = match mode {
                Mode::Read | Mode::ReadSubtract => {
                    let tmp: T =
                        unsafe { mem::transmute_copy(self.data.get_unchecked(pos as usize)) };
                    num::cast(tmp).unwrap_or(0)
                }
                _ => pos,
            };

            pos = match mode {
                Mode::Nop => pos + self.base as isize,
                Mode::ReadSubtract => pos - self.base as isize,
                _ => pos,
            };

            Some((pos + extra as isize) as usize)
        })
    }
}

pub fn get(name: &str, pid: u32, handle: &HANDLE) -> Option<Module> {
    if name.is_empty() || pid == 0 || handle.is_null() {
        return None;
    }

    let snapshot = SnapshotHandle::new(pid, TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32)?;

    let mut remote_module: MODULEENTRY32W = unsafe { mem::zeroed() };
    remote_module.dwSize = mem::size_of::<MODULEENTRY32W>() as DWORD;

    if unsafe { Module32FirstW(*snapshot, &mut remote_module) } != 0 {
        loop {
            let s = String::from_utf16_lossy(&remote_module.szModule)
                .trim_matches('\0')
                .to_string();
            if name == s {
                let mut buffer = Module::new();
                buffer.name = s;
                buffer.base = remote_module.modBaseAddr as usize;
                buffer.size = remote_module.modBaseSize as usize;
                buffer.data.resize(buffer.size, 0u8);
                unsafe {
                    ReadProcessMemory(
                        *handle,
                        buffer.base as LPCVOID,
                        buffer.data.as_mut_ptr() as LPVOID,
                        buffer.size as SIZE_T,
                        ptr::null_mut::<SIZE_T>(),
                    );
                }
                return Some(buffer);
            }
            if unsafe { Module32NextW(*snapshot, &mut remote_module) } == FALSE {
                break;
            }
        }
    }
    None
}
