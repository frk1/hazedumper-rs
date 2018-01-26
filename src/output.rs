extern crate chrono;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

use self::chrono::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

type Map<T> = BTreeMap<String, T>;

// This struct represents the dumper results.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Results {
    // Timestamp.
    #[serde(default = "Local::now")]
    pub timestamp: DateTime<Local>,

    // Results of the signature scanning.
    #[serde(default)]
    pub signatures: Map<usize>,

    // Optional results for the netvar scanning.
    // Will not be serialized if `None`.
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
        let mut out_json = File::create(format!("{}.json", filename))?;
        let mut out_min_json = File::create(format!("{}.min.json", filename))?;
        let mut out_yaml = File::create(format!("{}.yaml", filename))?;
        let mut out_toml = File::create(format!("{}.toml", filename))?;

        serde_json::to_writer_pretty(&mut out_json, self).unwrap();
        serde_json::to_writer(&mut out_min_json, self).unwrap();
        serde_yaml::to_writer(&mut out_yaml, self).unwrap();
        out_toml
            .write_all(toml::ser::to_string_pretty(self).unwrap().as_bytes())
            .unwrap();
        self.dump_hpp(filename)?;

        Ok(())
    }

    fn dump_hpp(&self, filename: &str) -> ::std::io::Result<()> {
        let mut f = File::create(format!("{}.hpp", filename))?;
        writeln!(&mut f, "#pragma once")?;
        writeln!(&mut f, "#include <cstddef>\n")?;
        writeln!(&mut f, "// {}\n", self.timestamp)?;

        writeln!(&mut f, "namespace hazedumper {{")?;
        if let Some(ref netvars) = self.netvars {
            writeln!(&mut f, "namespace netvars {{")?;
            for (k, v) in netvars {
                writeln!(&mut f, "constexpr ::std::ptrdiff_t {} = {:#X};", k, v)?;
            }
            writeln!(&mut f, "}} // namespace netvars")?;
        }

        writeln!(&mut f, "namespace signatures {{")?;
        for (k, v) in &self.signatures {
            writeln!(&mut f, "constexpr ::std::ptrdiff_t {} = {:#X};", k, v)?;
        }
        writeln!(&mut f, "}} // namespace signatures")?;
        writeln!(&mut f, "}} // namespace hazedumper")?;
        Ok(())
    }
}
