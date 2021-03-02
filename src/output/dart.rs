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
        writeln!(&mut self.file, "library hazedumper;\n")?;
        self.timestamp()?;
        self.netvars()?;
        self.signatures()?;
        Ok(())
    }

    /// Write the header.
    fn header(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "// {}\n", self.res.timestamp)?;
        Ok(())
    }

    /// Write the timestamp.
    fn timestamp(&mut self) -> io::Result<()> {
        writeln!(
            &mut self.file,
            "final int timestamp = {};\n",
            self.res.timestamp.timestamp()
        )
    }

    /// Write the netvars.
    fn netvars(&mut self) -> io::Result<()> {
        if let Some(ref netvars) = self.res.netvars {
            writeln!(&mut self.file, "class netvars {{")?;
            for (k, v) in netvars {
                writeln!(
                    &mut self.file,
                    "    static final int {} = {:#X};",
                    k, v
                )?;
            }
            writeln!(&mut self.file, "}}")?;
        }
        Ok(())
    }

    /// Write the signatures.
    fn signatures(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "class signatures {{")?;
        for (k, v) in &self.res.signatures {
            writeln!(
                &mut self.file,
                "    static final int {} = {:#X};",
                k, v
            )?;
        }
        writeln!(&mut self.file, "}}")?;
        Ok(())
    }
}

impl<'a> Dumper<'a> {
    /// Create new instance.
    pub fn new(res: &'a Results, name: &str) -> io::Result<Self> {
        let f = File::create(format!("{}.dart", name))?;
        Ok(Dumper { res, file: f })
    }
}
