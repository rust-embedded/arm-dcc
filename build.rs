use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    if target.starts_with("armv") {
        fs::copy(format!("bin/{}.a", target), out_dir.join("libdcc.a")).unwrap();
        println!("cargo:rustc-link-lib=static=dcc");
        println!("cargo:rustc-link-search={}", out_dir.display());
    }

    println!("cargo:rerun-if-changed=bin/armebv7r-none-eabi.a");
    println!("cargo:rerun-if-changed=bin/armebv7r-none-eabi.a");
    println!("cargo:rerun-if-changed=bin/armv7r-none-eabihf.a");
    println!("cargo:rerun-if-changed=bin/armv7r-none-eabihf.a");
    println!("cargo:rerun-if-changed=build.rs");
}
