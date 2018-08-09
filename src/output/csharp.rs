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

use std::fs::File;
use std::io;
use std::io::prelude::*;

use super::{Dumpable, Results};

pub struct Dumper<'a> {
    res: &'a Results,
    file: File,
}

impl<'a> Dumpable for Dumper<'a> {
    fn dump(&mut self) -> io::Result<()> {
        self.header()?;
        writeln!(&mut self.file, "namespace hazedumper\n{{")?;
        self.timestamp()?;
        self.netvars()?;
        self.signatures()?;
        writeln!(&mut self.file, "}} // namespace hazedumper")?;
        Ok(())
    }

    /// Write the header.
    fn header(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "using System;\n")?;
        writeln!(&mut self.file, "// {}\n", self.res.timestamp)?;
        Ok(())
    }

    /// Write the timestamp.
    fn timestamp(&mut self) -> io::Result<()> {
        writeln!(
            &mut self.file,
            "    public const Int32 timestamp = {};",
            self.res.timestamp.timestamp()
        )
    }

    /// Write the netvars.
    fn netvars(&mut self) -> io::Result<()> {
        if let Some(ref netvars) = self.res.netvars {
            writeln!(&mut self.file, "    public static class netvars\n    {{")?;
            for (k, v) in netvars {
                writeln!(
                    &mut self.file,
                    "        public const Int32 {} = {:#X};",
                    k, v
                )?;
            }
            writeln!(&mut self.file, "    }}")?;
        }
        Ok(())
    }

    /// Write the signatures.
    fn signatures(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "    public static class signatures\n    {{")?;
        for (k, v) in &self.res.signatures {
            writeln!(
                &mut self.file,
                "        public const Int32 {} = {:#X};",
                k, v
            )?;
        }
        writeln!(&mut self.file, "    }}")?;
        Ok(())
    }
}

impl<'a> Dumper<'a> {
    /// Create new instance.
    pub fn new(res: &'a Results, name: &str) -> io::Result<Self> {
        let f = File::create(format!("{}.cs", name))?;
        Ok(Dumper { res, file: f })
    }
}
