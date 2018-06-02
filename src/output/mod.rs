extern crate chrono;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

mod csharp;
mod hpp;
mod vbnet;
mod rust;

use self::chrono::prelude::*;
use self::chrono::serde::ts_seconds;
use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub type Map<T> = BTreeMap<String, T>;

// This struct represents the dumper results.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Results {
    // Timestamp.
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,

    // Results of the signature scanning.
    #[serde(default)]
    pub signatures: Map<usize>,

    // Optional results for the netvar scanning.
    // Will not be serialized if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub netvars: Option<Map<isize>>,
}

/// Trait to be implemented to be dumpable.
trait Dumpable {
    /// Dump the results.
    fn dump(&mut self) -> io::Result<()>;

    /// Write the header.
    fn header(&mut self) -> io::Result<()>;

    /// Write the netvars.
    fn netvars(&mut self) -> io::Result<()>;

    /// Write the signatures.
    fn signatures(&mut self) -> io::Result<()>;
}

impl Results {
    pub fn new(signatures: Map<usize>, netvars: Option<Map<isize>>) -> Self {
        Results {
            timestamp: Utc::now(),
            signatures,
            netvars,
        }
    }

    pub fn dump_all(&self, filename: &str) -> ::std::io::Result<()> {
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

        let mut out_hpp = hpp::Dumper::new(self, filename)?;
        let mut out_csharp = csharp::Dumper::new(self, filename)?;
        let mut out_vbnet = vbnet::Dumper::new(self, filename)?;
        let mut out_rust = rust::Dumper::new(self, filename)?;
        out_hpp.dump()?;
        out_csharp.dump()?;
        out_vbnet.dump()?;
        out_rust.dump()?;

        Ok(())
    }
}
