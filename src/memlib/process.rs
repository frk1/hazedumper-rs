extern crate winapi;

use std::{mem, ptr};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use self::winapi::shared::basetsd::SIZE_T;
use self::winapi::shared::minwindef::{BOOL, FALSE, LPCVOID, LPVOID, PBOOL, TRUE};
use self::winapi::shared::ntdef::HANDLE;
use self::winapi::um::handleapi::CloseHandle;
use self::winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use self::winapi::um::processthreadsapi::OpenProcess;
use self::winapi::um::tlhelp32::{PROCESSENTRY32W,
                                 Process32FirstW,
                                 Process32NextW,
                                 TH32CS_SNAPPROCESS};
use self::winapi::um::winnt::PROCESS_ALL_ACCESS;
use self::winapi::um::wow64apiset::IsWow64Process;
use memlib::*;

impl Constructor for PROCESSENTRY32W {
    /// Create a new instance of `PROCESSENTRY32W`
    fn new() -> Self {
        let mut pe: PROCESSENTRY32W = unsafe { mem::zeroed() };
        pe.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        pe
    }
}

#[derive(Debug)]
pub struct Process {
    // Process id.
    pub id: u32,

    // Process bitness.
    pub is_wow64: bool,

    // Process `HANDLE`.
    handle: HANDLE,

    // List of modules.
    modules: RefCell<HashMap<String, Rc<super::module::Module>>>,
}

impl Process {
    pub fn read<T: Copy>(&self, address: usize) -> Option<T> {
        let mut buffer = unsafe { mem::zeroed::<T>() };
        match unsafe {
            ReadProcessMemory(
                self.handle,
                address as LPCVOID,
                &mut buffer as *mut T as LPVOID,
                mem::size_of::<T>() as SIZE_T,
                ptr::null_mut::<SIZE_T>(),
            )
        } {
            TRUE => Some(buffer),
            _ => None,
        }
    }

    pub fn read_ptr<T: Copy>(&self, buf: *mut T, address: usize, count: usize) -> bool {
        unsafe {
            ReadProcessMemory(
                self.handle,
                address as LPCVOID,
                buf as *mut T as LPVOID,
                mem::size_of::<T>() as SIZE_T * count,
                ptr::null_mut::<SIZE_T>(),
            ) == TRUE
        }
    }

    pub fn write<T: Copy>(&self, address: u32, buf: &T) -> bool {
        unsafe {
            WriteProcessMemory(
                self.handle,
                address as LPVOID,
                buf as *const T as LPCVOID,
                mem::size_of::<T>() as SIZE_T,
                ptr::null_mut::<SIZE_T>(),
            ) == TRUE
        }
    }
}

impl Process {
    pub fn get_module(&self, name: &str) -> Option<Rc<super::module::Module>> {
        let mut b = self.modules.borrow_mut();
        if b.contains_key(name) {
            return b.get(name).cloned();
        }

        super::module::get(name, self).and_then(|m| b.insert(name.to_string(), Rc::new(m)));
        b.get(name).cloned()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { CloseHandle(self.handle) };
        }
    }
}

pub fn from_pid(pid: u32) -> Option<Process> {
    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };
    if handle.is_null() {
        return None;
    }

    let mut tmp: BOOL = 0;

    if unsafe { IsWow64Process(handle, &mut tmp as PBOOL) } == FALSE {
        warn!("Could not determine process bitness: IsWow64Process returned an error!");
        return None;
    }

    let is_wow64 = match tmp {
        FALSE => false,
        _ => true,
    };
    debug!("PID {} is_wow64: {}", pid, is_wow64);

    Some(Process {
        id: pid,
        is_wow64: is_wow64,
        handle: handle,
        modules: RefCell::new(HashMap::new()),
    })
}

/// Wrapper around the `Process32FirstW` windows api
fn process32_first(h: &SnapshotHandle, pe: &mut PROCESSENTRY32W) -> bool {
    unsafe { Process32FirstW(**h, pe) == TRUE }
}

/// Wrapper around the `Process32NextW` windows api
fn process32_next(h: &SnapshotHandle, pe: &mut PROCESSENTRY32W) -> bool {
    unsafe { Process32NextW(**h, pe) == TRUE }
}

pub fn from_name(name: &str) -> Option<Process> {
    let snapshot = SnapshotHandle::new(0, TH32CS_SNAPPROCESS)?;
    let mut pe = PROCESSENTRY32W::new();

    if !process32_first(&snapshot, &mut pe) {
        return None;
    }

    loop {
        let pname = String::from_utf16(&pe.szExeFile).unwrap_or_else(|_| String::new());
        if pname.contains(name) {
            return from_pid(pe.th32ProcessID);
        }
        if !process32_next(&snapshot, &mut pe) {
            break;
        }
    }

    None
}
