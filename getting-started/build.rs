use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let target = PathBuf::from(out_dir).join("memory.x");

    fs::copy("memory.x", target).unwrap();

    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
}
