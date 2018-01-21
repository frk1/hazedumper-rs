extern crate chrono;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

use self::chrono::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

type Map<T> = BTreeMap<String, T>;

#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum ConfigError {
    #[fail(display = "Invalid Bitness, try 'X86' or 'X64'")] InvalidBitness,
    #[fail(display = "Could not load config from file")] LoadingFromFile,
}

// This struct represents a signature.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Results {
    // Timestamp.
    #[serde(default = "Local::now")]
    pub timestamp: DateTime<Local>,

    // Signature offsets for dereferencing.
    #[serde(default)]
    pub signatures: Map<usize>,

    // Extra to be added to the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub netvars: Option<Map<isize>>,
}

impl Results {
    pub fn new(signatures: Map<usize>, netvars: Option<Map<isize>>) -> Self {
        Results {
            timestamp: Local::now(),
            signatures,
            netvars,
        }
    }

    pub fn dump(&self, filename: &str) -> ::std::io::Result<()> {
        let mut out = File::create(format!("{}.json", filename))?;
        serde_json::to_writer_pretty(&mut out, self).unwrap();
        let mut out = File::create(format!("{}.min.json", filename))?;
        serde_json::to_writer(&mut out, self).unwrap();

        let mut out = File::create(format!("{}.yaml", filename))?;
        serde_yaml::to_writer(&mut out, self).unwrap();

        let mut out = File::create(format!("{}.toml", filename))?;
        let ser = toml::ser::to_string_pretty(self).unwrap();
        out.write_all(&ser.as_bytes()).unwrap();

        self.dump_hpp(filename)?;
        Ok(())
    }

    fn dump_hpp(&self, filename: &str) -> ::std::io::Result<()> {
        let mut f = File::create(format!("{}.hpp", filename))?;
        writeln!(&mut f, "#pragma once")?;
        writeln!(&mut f, "#include <cstddef>\n")?;
        writeln!(&mut f, "// {}\n", Local::now().to_string())?;

        writeln!(&mut f, "namespace hazedumper {{")?;
        if let Some(ref netvars) = self.netvars {
            writeln!(&mut f, "namespace netvars {{")?;
            for (k, v) in netvars {
                writeln!(&mut f, "constexpr ::std::ptrdiff_t {} = 0x{:X};", k, v)?;
            }
            writeln!(&mut f, "}} // namespace netvars")?;
        }

        writeln!(&mut f, "namespace signatures {{")?;
        for (k, v) in &self.signatures {
            writeln!(&mut f, "constexpr ::std::ptrdiff_t {} = 0x{:X};", k, v)?;
        }
        writeln!(&mut f, "}} // namespace signatures")?;
        writeln!(&mut f, "}} // namespace hazedumper")?;
        Ok(())
    }
}
