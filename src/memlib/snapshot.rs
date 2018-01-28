extern crate winapi;
use self::winapi::shared::ntdef::HANDLE;
use self::winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use self::winapi::um::tlhelp32::CreateToolhelp32Snapshot;
use std::ops::Deref;

/// Wrapper around the windows `HANDLE` returned from
/// `kernel32::CreateToolhelp32Snapshot`.
pub struct SnapshotHandle {
    pub handle: HANDLE,
}

impl SnapshotHandle {
    /// Constructs a new `SnapshotHandle`.
    ///
    /// Calls the `kernel32::CreateToolhelp32Snapshot` windows api.
    pub fn new(pid: u32, flags: u32) -> Option<Self> {
        let handle = unsafe { CreateToolhelp32Snapshot(flags, pid) };
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return None;
        }

        Some(SnapshotHandle { handle })
    }
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Deref for SnapshotHandle {
    type Target = HANDLE;

    fn deref(&self) -> &HANDLE {
        &self.handle
    }
}
