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
        writeln!(&mut self.file, "namespace hazedumper {{")?;
        self.netvars()?;
        self.signatures()?;
        writeln!(&mut self.file, "}} // namespace hazedumper")?;
        Ok(())
    }

    /// Write the header.
    fn header(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "#pragma once")?;
        writeln!(&mut self.file, "#include <cstddef>\n")?;
        writeln!(&mut self.file, "// {}\n", self.res.timestamp)?;
        Ok(())
    }

    /// Write the netvars.
    fn netvars(&mut self) -> io::Result<()> {
        if let Some(ref netvars) = self.res.netvars {
            writeln!(&mut self.file, "namespace netvars {{")?;
            for (k, v) in netvars {
                writeln!(
                    &mut self.file,
                    "constexpr ::std::ptrdiff_t {} = {:#X};",
                    k, v
                )?;
            }
            writeln!(&mut self.file, "}} // namespace netvars")?;
        }
        Ok(())
    }

    /// Write the signatures.
    fn signatures(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "namespace signatures {{")?;
        for (k, v) in &self.res.signatures {
            writeln!(
                &mut self.file,
                "constexpr ::std::ptrdiff_t {} = {:#X};",
                k, v
            )?;
        }
        writeln!(&mut self.file, "}} // namespace signatures")?;
        Ok(())
    }
}

impl<'a> Dumper<'a> {
    /// Create new instance.
    pub fn new(res: &'a Results, name: &str) -> io::Result<Self> {
        let f = File::create(format!("{}.hpp", name))?;
        Ok(Dumper { res: res, file: f })
    }
}
