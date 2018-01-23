extern crate serde_json;

use std::fs::File;
use std::str::FromStr;

use memlib::Bitness;

#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum ConfigError {
    #[fail(display = "Invalid Bitness, try 'X86' or 'X64'")] InvalidBitness,
    #[fail(display = "Could not load config from file")] LoadingFromFile,
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

impl FromStr for Bitness {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "x86" | "X86" => Ok(Bitness::X86),
            "x64" | "X64" => Ok(Bitness::X64),
            _ => Err(ConfigError::InvalidBitness),
        }
    }
}

// This struct represents the config.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // Executable target name.
    pub executable: String,

    // `Bitness` of the target process. Defaults to X86.
    #[serde(default)]
    pub bitness: Bitness,

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
            bitness: Bitness::X86,
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
