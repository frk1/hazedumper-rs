extern crate winapi;

use std::{mem, ptr};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use self::winapi::shared::basetsd::SIZE_T;
use self::winapi::shared::minwindef::{FALSE, LPCVOID, LPVOID, TRUE};
use self::winapi::shared::ntdef::HANDLE;
use self::winapi::um::handleapi::CloseHandle;
use self::winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use self::winapi::um::processthreadsapi::OpenProcess;
use self::winapi::um::tlhelp32::{PROCESSENTRY32W, Process32FirstW, Process32NextW,
                                 TH32CS_SNAPPROCESS};
use self::winapi::um::winnt::PROCESS_ALL_ACCESS;

use snapshot::SnapshotHandle;

#[derive(Debug)]
pub struct Process {
    pub id: u32,
    handle: HANDLE,
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

        super::module::get(name, self.id, &self.handle)
            .and_then(|m| b.insert(name.to_string(), Rc::new(m)));

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
    Some(Process {
        id: pid,
        handle: handle,
        modules: RefCell::new(HashMap::new()),
    })
}

pub fn from_name(name: &str) -> Option<Process> {
    let snapshot = SnapshotHandle::new(0, TH32CS_SNAPPROCESS)?;
    let mut process: PROCESSENTRY32W = unsafe { mem::zeroed() };
    process.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

    if unsafe { Process32FirstW(*snapshot, &mut process) } == FALSE {
        return None;
    }

    loop {
        let pn = String::from_utf16(&process.szExeFile).unwrap_or_else(|_| String::new());
        if pn.contains(name) {
            return from_pid(process.th32ProcessID);
        }
        if unsafe { Process32NextW(*snapshot, &mut process) } == FALSE {
            break;
        }
    }

    None
}
