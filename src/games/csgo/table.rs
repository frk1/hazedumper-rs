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

#![allow(clippy::double_parens)]

use crate::memlib::Module;
use nom::*;
use std::str;

use super::prop::{RecvProp, RecvPropIterator};

#[derive(Debug, Clone, PartialEq)]
pub struct RecvTable {
    pub name: String,
    pub props: Vec<RecvProp>,
}

impl RecvTable {
    // offset_name, offset_props, num_props
    #[rustfmt::skip]
    named!(
        parse_raw<(usize, usize, usize)>,
        do_parse!(
            offset_props: le_u32 >>
            num_props: le_u32    >>
            take!(4)             >>
            offset_name: le_u32  >>
            ((
                offset_name as usize,
                offset_props as usize,
                num_props as usize,
            ))
        )
    );

    pub fn parse(base: usize, module: &Module) -> Option<Self> {
        trace!("Starting to parse RecvTable at {:#x}", base);
        if base == 0 {
            return None;
        }

        let data = module.get_slice(base, 0x10, false)?;
        let (_, (offset_name, offset_props, num_props)) = Self::parse_raw(&data).ok()?;

        let name = crate::helpers::parse_string(module.get(offset_name, false)?)
            .ok()?
            .1
            .to_string();
        trace!("Found RecvTable '{}' at {:#x}", name, base);

        Some(Self {
            name,
            props: RecvPropIterator::new(offset_props, num_props, module).collect::<Vec<_>>(),
        })
    }

    pub fn get_offset(&self, name: &str) -> Option<i32> {
        for prop in &self.props {
            if let Some(o) = prop.get_offset(name) {
                return Some(o);
            }
        }
        None
    }
}
