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

extern crate serde_json;

use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum ConfigError {
    #[fail(display = "Could not load config from file")]
    LoadingFromFile,
}

pub type Result<T> = ::std::result::Result<T, ConfigError>;

// This struct represents a signature.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Signature {
    // Signature name.
    pub name: String,

    // Signature pattern.
    pub pattern: String,

    // Module name.
    pub module: String,

    // Signature offsets for dereferencing.
    #[serde(default)]
    pub offsets: Vec<isize>,

    // Extra to be added to the result.
    #[serde(default)]
    pub extra: isize,

    // If true, subtract module base from result.
    #[serde(default)]
    pub relative: bool,

    // If true, read a u32 at the position and add it to the result.
    #[serde(default)]
    pub rip_relative: bool,

    // Offset to the rip relative.
    #[serde(default)]
    pub rip_offset: isize,
}

// This struct represents a netvar.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Netvar {
    // Netvar name.
    pub name: String,

    // Table name.
    pub table: String,

    // Prop name.
    pub prop: String,

    // Offset to be added to the result.
    #[serde(default)]
    pub offset: isize,
}

impl Default for Signature {
    fn default() -> Self {
        Signature {
            name: "".to_string(),
            pattern: "".to_string(),
            module: "".to_string(),
            offsets: vec![],
            extra: 0,
            relative: false,
            rip_relative: false,
            rip_offset: 0,
        }
    }
}

// This struct represents the config.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // Executable target name.
    pub executable: String,

    // Output file names
    #[serde(default)]
    pub filename: String,

    // `Vec` containing the `Signature`s.
    #[serde(default)]
    pub signatures: Vec<Signature>,

    // `Vec` containing the `Netvar`s.
    #[serde(default)]
    pub netvars: Vec<Netvar>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            executable: "csgo.exe".to_string(),
            filename: "csgo".to_string(),
            signatures: vec![],
            netvars: vec![],
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let mut file_input = File::open(path).map_err(|_| ConfigError::LoadingFromFile)?;
        serde_json::from_reader(&mut file_input).map_err(|_| ConfigError::LoadingFromFile)
    }
}
