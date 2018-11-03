// MIT License
//
// Copyright (c) 2018 frk <hazefrk+dev@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate winapi;

use self::winapi::shared::minwindef::FALSE;
use self::winapi::um::tlhelp32::{
    Module32FirstW,
    Module32NextW,
    MODULEENTRY32W,
    TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};
use crate::memlib::*;
use std::mem;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub data: Vec<u8>,
}

impl Constructor for MODULEENTRY32W {
    /// Create a new instance of `MODULEENTRY32W`
    fn new() -> Self {
        let mut module: MODULEENTRY32W = unsafe { mem::zeroed() };
        module.dwSize = mem::size_of::<MODULEENTRY32W>() as u32;
        module
    }
}

impl Module {
    fn from_module_entry(me: &MODULEENTRY32W, name: &str, process: &Process) -> Option<Self> {
        let mut i = Module {
            name: name.to_string(),
            base: me.modBaseAddr as usize,
            size: me.modBaseSize as usize,
            data: vec![0u8; me.modBaseSize as usize],
        };

        if process.read_ptr(i.data.as_mut_ptr(), i.base, i.size) {
            return Some(i);
        }

        None
    }

    pub fn find_pattern(&self, pattern: &str) -> Option<usize> {
        findpattern::find_pattern(&self.data, pattern)
    }

    /// o: Offset
    /// is_relative: Base has already been subtracted.
    pub fn get_raw<T: Copy>(&self, mut o: usize, is_relative: bool) -> Option<T> {
        if !is_relative {
            o -= self.base;
        }
        if o + mem::size_of::<T>() >= self.data.len() {
            return None;
        }
        let ptr = self.data.get(o)?;
        let raw: T = unsafe { mem::transmute_copy(ptr) };
        Some(raw)
    }

    /// is_relative: if true, the base has already been subtracted.
    pub fn get_slice(&self, mut offset: usize, len: usize, is_relative: bool) -> Option<&[u8]> {
        if !is_relative {
            offset = offset.wrapping_sub(self.base);
        }
        self.data.get(offset..(offset + len))
    }

    /// is_relative: if true, the base has already been subtracted.
    pub fn get(&self, mut offset: usize, is_relative: bool) -> Option<&[u8]> {
        if !is_relative {
            offset = offset.wrapping_sub(self.base);
        }
        self.data.get(offset..)
    }
}

/// Wrapper around the `Module32FirstW` windows api
fn module32_first(h: &SnapshotHandle, me: &mut MODULEENTRY32W) -> bool {
    unsafe { Module32FirstW(**h, me) != FALSE }
}

/// Wrapper around the `Module32NextW` windows api
fn module32_next(h: &SnapshotHandle, me: &mut MODULEENTRY32W) -> bool {
    unsafe { Module32NextW(**h, me) != FALSE }
}

pub fn get(name: &str, process: &Process) -> Option<Module> {
    let snapshot = SnapshotHandle::new(process.id, TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32)?;
    let mut me = MODULEENTRY32W::new();

    if !module32_first(&snapshot, &mut me) {
        return None;
    }

    loop {
        let s = String::from_utf16_lossy(&me.szModule)
            .trim_matches('\0')
            .to_string();

        if name == s {
            return Module::from_module_entry(&me, &s, process);
        }

        if !module32_next(&snapshot, &mut me) {
            break;
        }
    }

    None
}
