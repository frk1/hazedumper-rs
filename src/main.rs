#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simplelog;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

mod config;
mod netvars;
mod memlib;
mod sigscan;

use std::fs::OpenOptions;
use std::process::exit;

use config::Config;
use simplelog::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hazedumper", about = "Signature scanning for every game!", version = "2.0.0",
            author = "frk <hazefrk+dev@gmail.com>")]
struct Opt {
    /// A flag, true if used in the command line.
    #[structopt(short = "v", help = "Activate verbose mode")]
    verbose: u64,

    /// Optional parameter, the config file.
    #[structopt(help = "Path to the config file (default: config.json)")]
    config: Option<String>,

    /// Optional parameter, overrides the target executable.
    #[structopt(short = "t", long = "target", help = "Target executable")]
    target: Option<String>,

    /// Optional parameter, overrides the target bitness.
    #[structopt(short = "b", long = "bitness", help = "Target bitness (X86 or X64)")]
    bitness: Option<config::Bitness>,
}

fn setup_log(v: u64) -> () {
    let level = match v {
        0 => LogLevelFilter::Info,
        _ => LogLevelFilter::Debug,
    };

    let logfile = OpenOptions::new()
        .append(true)
        .create(true)
        .open("hazedumper.log");

    CombinedLogger::init(vec![
        TermLogger::new(level, simplelog::Config::default()).unwrap(),
        WriteLogger::new(level, simplelog::Config::default(), logfile.unwrap()),
    ]).unwrap();
}

fn main() {
    let opt = Opt::from_args();
    setup_log(opt.verbose);

    info!("Loading config");
    let conf = Config::load(&opt.config.unwrap_or("config.json".to_string()))
        .unwrap_or_else(|_| Config::default());

    info!("Opening target process: {}", conf.executable);
    let process = memlib::from_name(&conf.executable)
        .ok_or_else(|| {
            error!("Could not open process {}!", conf.executable);
            exit(1);
        })
        .unwrap();

    for (i, sig) in conf.signatures.iter().enumerate() {
        if let Err(err) = sigscan::find_signature32(&sig, &process) {
            warn!("{} sigscan failed: {}", sig.name, err);
        }
    }
}
