#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt_derive;

extern crate simplelog;
extern crate structopt;

mod config;
mod games;
mod memlib;
mod output;
mod sigscan;

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::process::exit;

use config::Config;
use simplelog::*;
use structopt::StructOpt;

type Map<T> = BTreeMap<String, T>;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "hazedumper",
    about = "Signature scanning for every game!",
    author = "frk <hazefrk+dev@gmail.com>"
)]
struct Opt {
    /// A flag, true if used in the command line.
    #[structopt(short = "v", help = "Vverbose mode", parse(from_occurrences))]
    verbose: u8,

    /// A flag, true if used in the command line.
    #[structopt(short = "s", long = "silent")]
    silent: bool,

    /// Optional parameter, the config file.
    #[structopt(short = "c", long = "config", help = "Config file [config.json]")]
    config: Option<String>,

    /// Optional parameter, the config file.
    #[structopt(short = "o", long = "output", help = "Output filename")]
    filename: Option<String>,

    /// Optional parameter, overrides the target executable.
    #[structopt(short = "t", long = "target", help = "Process name")]
    target: Option<String>,
}

fn main() {
    let app = Opt::clap().version(env!("GIT_PKG_VERSION_SEMVER"));
    let opt = Opt::from_clap(&app.get_matches());
    if !opt.silent {
        setup_log(opt.verbose);
    }

    let conf_path = opt.config.unwrap_or_else(|| "config.json".to_string());
    debug!("Loading config: {}", conf_path);
    let conf = Config::load(&conf_path).unwrap_or_default();

    info!("Opening target process: {}", conf.executable);
    let process = memlib::from_name(&conf.executable)
        .ok_or_else(|| {
            error!("Could not open process {}!", conf.executable);
            exit(1);
        })
        .unwrap();

    let sigs = scan_signatures(&conf, &process);
    let netvars = match conf.executable.as_ref() {
        "csgo.exe" => scan_netvars(&sigs, &conf, &process),
        _ => None,
    };

    let results = output::Results::new(sigs, netvars);
    let filename = opt.filename.unwrap_or_else(|| "csgo".to_string());
    results.dump_all(&filename).expect("Dump results");
}

/// Setup log levels for terminal and file.
fn setup_log(v: u8) -> () {
    use LevelFilter::{Debug, Info, Trace};
    let (level_term, level_file) = match v {
        0 => (Info, Info),
        1 => (Debug, Debug),
        _ => (Debug, Trace),
    };

    let logfile = OpenOptions::new()
        .append(true)
        .create(true)
        .open("hazedumper.log");

    CombinedLogger::init(vec![
        TermLogger::new(level_term, simplelog::Config::default()).unwrap(),
        WriteLogger::new(level_file, simplelog::Config::default(), logfile.unwrap()),
    ]).unwrap();
}

/// Scan the signatures from the config and return a `Map<usize>`.
fn scan_signatures(conf: &Config, process: &memlib::Process) -> Map<usize> {
    info!(
        "Starting signature scanning: {} items",
        conf.signatures.len()
    );
    let mut res = BTreeMap::new();

    for sig in &conf.signatures {
        match sigscan::find_signature(sig, process) {
            Ok(r) => {
                res.insert(sig.name.clone(), r);
                info!("Found signature: {} => {:#X}", sig.name, r);
            }
            Err(err) => warn!("{} sigscan failed: {}", sig.name, err),
        };
    }

    info!(
        "Finished signature scanning: {}/{} items successful",
        res.len(),
        conf.signatures.len()
    );
    res
}

/// Scan the netvars from the config and return a `Option<Map<i32>>`.
fn scan_netvars(sigs: &Map<usize>, conf: &Config, process: &memlib::Process) -> Option<Map<isize>> {
    info!("Starting netvar scanning: {} items", conf.netvars.len());

    let first = sigs.get("dwGetAllClasses")?;
    let netvars = games::csgo::NetvarManager::new(*first, process)?;

    let mut res = BTreeMap::new();
    for netvar in &conf.netvars {
        match netvars.get_offset(&netvar.table, &netvar.prop) {
            Some(o) => {
                res.insert(netvar.name.clone(), o as isize + netvar.offset);
                info!("Found netvar: {} => {:#X}", netvar.name, o);
            }
            None => warn!("{} netvar failed!", netvar.name),
        };
    }

    info!(
        "Finished netvar scanning: {}/{} items successful",
        res.len(),
        conf.netvars.len()
    );
    Some(res)
}
