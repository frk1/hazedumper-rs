extern crate chrono;
extern crate json;
extern crate memlib;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;

mod csgo;

use chrono::*;
use json::*;
use memlib::process::Process;

const DEFAULT_CONFIG: &str = include_str!("default.json");

fn main() {
    println!("> -------------------------------- <");
    println!("> Hazedumper v1.0.0 - Rust version <");
    println!(">   https://haze-productions.com   <");
    println!("> -------------------------------- <\n");

    let data = scan_game().unwrap();

    dump_json("csgo", &data);
    dump_hpp("csgo", &data).expect("Could not dump .hpp");
    dump_toml("csgo", &data).expect("Could not dump .toml");
    dump_csharp("csgo", &data).expect("Could not dump .cs");
    dump_vbnet("csgo", &data).expect("Could not dump .vb");

    println!("\n> Done. Press any key to exit.");

    let mut stdin = std::io::stdin();
    let input = &mut [0u8];
    assert!(stdin.read(input).is_ok());
}

fn scan_game() -> Option<JsonValue> {
    let mut config = JsonValue::new_object();
    load_config(&mut config).unwrap_or_else(|_| write_default_config(&mut config));

    let process = memlib::process::from_name("csgo.exe").expect("Error! Could not open csgo.exe!");
    let mut data = JsonValue::new_object();

    data["timestamp"] = Local::now().timestamp().into();
    data["modules"]["client.dll"]["size"] = process.get_module("client.dll")?.size.into();
    data["modules"]["engine.dll"]["size"] = process.get_module("engine.dll")?.size.into();

    println!("--- Signatures ---");
    scan_signatures(&config, &mut data, &process);

    println!("\n--- Netvars ---");
    match data["signatures"]["dwGetAllClasses"].as_usize() {
        Some(first_class) => {
            scan_netvars(&config, &mut data, &process, first_class);
        }
        None => {
            println!("\n - You need a working 'dwGetAllClasses' signature to dump netvars! -\n");
        }
    }

    Some(data)
}

fn scan_signatures(config: &JsonValue, data: &mut JsonValue, process: &Process) -> Option<()> {
    use memlib::module::Mode;
    let get_bool = |j: &JsonValue, p: &str| j[p].as_bool().unwrap_or(false);

    let find_sig = |v: &JsonValue| -> Option<usize> {
        let module = process.get_module(v["module"].as_str()?)?;
        let m_read = get_bool(&v, "mode_read");
        let m_sub = get_bool(&v, "mode_subtract");

        let s_mode = match m_read {
            true if m_sub => Mode::ReadSubtract,
            true if !m_sub => Mode::Read,
            false if m_sub => Mode::Subtract,
            _ => Mode::Nop,
        };

        let s_pattern = v["pattern"].as_str()?;
        let s_offset = v["offset"].as_i32().unwrap_or(0);
        let s_extra = v["extra"].as_i32().unwrap_or(0);

        module.find_pattern::<u32>(s_pattern, s_mode, s_offset, s_extra)
    };

    for (k, v) in config["signatures"].entries() {
        match find_sig(v) {
            Some(addr) => {
                println!("{:<30} => 0x{:X}", k, addr);
                data["signatures"][k.to_string()] = addr.into();
            }
            None => println!("{:<30} => FAILED!", k),
        }
    }

    None
}

fn scan_netvars(
    config: &JsonValue,
    data: &mut JsonValue,
    process: &Process,
    first_class: usize,
) -> Option<()> {
    use csgo::netvars::NetvarManager;
    let first_class = first_class + process.get_module("client.dll")?.base;
    let manager = NetvarManager::load_netdata(&process, first_class)?;

    let get_recv_prop = |v: &JsonValue| -> Option<usize> {
        let table = v["table"].as_str()?;
        let netvar = v["netvar"].as_str()?;
        manager.get_recv_prop(&process, table, netvar)
    };

    for (k, v) in config["netvars"].entries() {
        let offset = v["offset"].as_i32().unwrap_or(0);

        match get_recv_prop(v) {
            Some(val) => {
                let val = (val as i32 + offset) as usize;
                println!("{:<30} => 0x{:X}", k, val);
                data["netvars"][k.to_string()] = val.into();
            }
            None => {
                println!("{:<30} => FAILED!", k);
            }
        }
    }

    None
}

fn load_config(j: &mut JsonValue) -> std::io::Result<()> {
    let mut f = File::open("config.json")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    *j = json::parse(&s).unwrap();
    Ok(())
}

fn write_default_config(j: &mut JsonValue) {
    *j = json::parse(DEFAULT_CONFIG).unwrap();
    File::create("config.json")
        .ok()
        .map_or((), |mut f| write!(&mut f, "{}", j.pretty(2)).unwrap_or(()));
}

fn dump_json(name: &str, j: &JsonValue) {
    File::create(format!("{}.json", name))
        .ok()
        .map_or((), |mut f| write!(&mut f, "{}", j.pretty(2)).unwrap_or(()));
    File::create(format!("{}.min.json", name))
        .ok()
        .map_or((), |mut f| write!(&mut f, "{}", j.dump()).unwrap_or(()));
}

fn dump_hpp(name: &str, j: &JsonValue) -> std::io::Result<()> {
    let mut f = File::create(format!("{}.hpp", name))?;
    writeln!(&mut f, "#pragma once")?;
    writeln!(&mut f, "#include <cstddef>\n")?;
    writeln!(&mut f, "// Offsets timestamp")?;
    writeln!(&mut f, "// {}\n", Local::now().to_rfc2822())?;

    writeln!(&mut f, "namespace hazedumper {{")?;
    writeln!(&mut f, "namespace netvars {{")?;
    for (key, val) in j["netvars"].entries() {
        writeln!(
            &mut f,
            "constexpr ::std::ptrdiff_t {} = 0x{:X};",
            key,
            val.as_u32().unwrap()
        )?;
    }
    writeln!(&mut f, "}} // namespace netvars")?;
    writeln!(&mut f, "namespace signatures {{")?;
    for (key, val) in j["signatures"].entries() {
        writeln!(
            &mut f,
            "constexpr ::std::ptrdiff_t {} = 0x{:X};",
            key,
            val.as_u32().unwrap()
        )?;
    }
    writeln!(&mut f, "}} // namespace signatures")?;
    writeln!(&mut f, "}} // namespace hazedumper")?;
    Ok(())
}

fn dump_csharp(name: &str, j: &JsonValue) -> std::io::Result<()> {
    let mut f = File::create(format!("{}.cs", name))?;
    writeln!(&mut f, "using System;\n")?;
    writeln!(&mut f, "// Offsets timestamp")?;
    writeln!(&mut f, "// {}\n", Local::now().to_rfc2822())?;

    writeln!(&mut f, "namespace hazedumper\n{{")?;
    writeln!(&mut f, "    public static class netvars\n    {{")?;
    for (key, val) in j["netvars"].entries() {
        writeln!(
            &mut f,
            "        public const Int32 {} = 0x{:X};",
            key,
            val.as_u32().unwrap()
        )?;
    }
    writeln!(&mut f, "    }}")?;
    writeln!(&mut f, "    public static class signatures\n    {{")?;
    for (key, val) in j["signatures"].entries() {
        writeln!(
            &mut f,
            "        public const Int32 {} = 0x{:X};",
            key,
            val.as_u32().unwrap()
        )?;
    }
    writeln!(&mut f, "    }}")?;
    writeln!(&mut f, "}} // namespace hazedumper")?;
    Ok(())
}

fn dump_vbnet(name: &str, j: &JsonValue) -> std::io::Result<()> {
    let mut f = File::create(format!("{}.vb", name))?;
    writeln!(&mut f, "' {}\n", Local::now().to_rfc2822())?;

    writeln!(&mut f, "Namespace hazedumper")?;
    writeln!(&mut f, "    Public Shared Class netvars")?;
    for (key, val) in j["netvars"].entries() {
        writeln!(
            &mut f,
            "        Public Const {} as Integer = &H{:X}",
            key,
            val.as_u32().unwrap()
        )?;
    }
    writeln!(&mut f, "    End Class")?;
    writeln!(&mut f, "    Public Shared Class signatures")?;
    for (key, val) in j["signatures"].entries() {
        writeln!(
            &mut f,
            "        Public Const {} as Integer = &H{:X}",
            key,
            val.as_u32().unwrap()
        )?;
    }
    writeln!(&mut f, "    End Class")?;
    writeln!(&mut f, "End Namespace")?;
    Ok(())
}

fn dump_toml(name: &str, j: &JsonValue) -> std::io::Result<()> {
    let mut t = toml::Table::new();
    let mut netvars = toml::Table::new();
    let mut signatures = toml::Table::new();
    let timestamp = Local::now();
    t.insert(
        "timestamp".to_string(),
        toml::Value::Integer(timestamp.timestamp()),
    );
    for (key, val) in j["netvars"].entries() {
        netvars.insert(key.to_string(), toml::Value::Integer(val.as_i64().unwrap()));
    }
    for (key, val) in j["signatures"].entries() {
        signatures.insert(key.to_string(), toml::Value::Integer(val.as_i64().unwrap()));
    }
    t.insert("netvars".to_string(), toml::encode(&netvars));
    t.insert("signatures".to_string(), toml::encode(&signatures));
    let mut f = File::create(format!("{}.toml", name))?;
    writeln!(&mut f, "# {}", timestamp.to_rfc2822())?;
    write!(&mut f, "{}", toml::encode_str(&t))
}
