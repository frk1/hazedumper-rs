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
        writeln!(&mut self.file, "pub mod hazedumper {{")?;
        self.netvars()?;
        self.signatures()?;
        writeln!(&mut self.file, "}}")?;
        Ok(())
    }

    /// Write the header.
    fn header(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "#![allow(warnings)]\n")?;
        writeln!(&mut self.file, "// {}\n", self.res.timestamp)?;
        Ok(())
    }

    /// Write the netvars.
    fn netvars(&mut self) -> io::Result<()> {
        if let Some(ref netvars) = self.res.netvars {
            writeln!(&mut self.file, "    pub mod netvars {{")?;
            for (k, v) in netvars {
                writeln!(
                    &mut self.file,
                    "        pub const {}: isize = {:#X};",
                    k, v
                )?;
            }
            writeln!(&mut self.file, "    }}")?;
        }
        Ok(())
    }

    /// Write the signatures.
    fn signatures(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "    pub mod signatures {{")?;
        for (k, v) in &self.res.signatures {
            writeln!(
                &mut self.file,
                "        pub const {}: isize = {:#X};",
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
        let f = File::create(format!("{}.rs", name))?;
        Ok(Dumper { res, file: f })
    }
}
