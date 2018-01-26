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
        writeln!(&mut self.file, "Namespace hazedumper")?;
        self.netvars()?;
        self.signatures()?;
        writeln!(&mut self.file, "End Namespace")?;
        Ok(())
    }

    /// Write the header.
    fn header(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "' {}\n", self.res.timestamp)?;
        Ok(())
    }

    /// Write the netvars.
    fn netvars(&mut self) -> io::Result<()> {
        if let Some(ref netvars) = self.res.netvars {
            writeln!(&mut self.file, "    Public Shared Class netvars")?;
            for (k, v) in netvars {
                writeln!(
                    &mut self.file,
                    "        Public Const {} as Integer = &H{:X}",
                    k, v
                )?;
            }
            writeln!(&mut self.file, "    End Class")?;
        }
        Ok(())
    }

    /// Write the signatures.
    fn signatures(&mut self) -> io::Result<()> {
        writeln!(&mut self.file, "    Public Shared Class signatures")?;
        for (k, v) in &self.res.signatures {
            writeln!(
                &mut self.file,
                "        Public Const {} as Integer = &H{:X}",
                k, v
            )?;
        }
        writeln!(&mut self.file, "    End Class")?;
        Ok(())
    }
}

impl<'a> Dumper<'a> {
    /// Create new instance.
    pub fn new(res: &'a Results, name: &str) -> io::Result<Self> {
        let f = File::create(format!("{}.vb", name))?;
        Ok(Dumper { res: res, file: f })
    }
}
