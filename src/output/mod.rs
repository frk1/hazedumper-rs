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

extern crate chrono;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

mod csharp;
mod hpp;
mod vbnet;

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
        out_hpp.dump()?;
        out_csharp.dump()?;
        out_vbnet.dump()?;

        Ok(())
    }
}
