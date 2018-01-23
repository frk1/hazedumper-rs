#[cfg(windows)]
extern crate winres;

use std::process::Command;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("app.ico")
        .set_language(0x0409)
        .set("FileDescription", &format!("HazeDumper {}", git_semver()));
    res.compile().unwrap();
}

fn git_semver() -> String {
    let output = Command::new("git")
        .args(&["describe", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[cfg(unix)]
fn main() {}
