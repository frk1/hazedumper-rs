#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("app.ico");
    res.set_language(0x0409);
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
