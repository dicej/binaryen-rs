extern crate bindgen;
extern crate cmake;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn gen_bindings() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // See https://github.com/rust-lang-nursery/rust-bindgen/issues/947
        .trust_clang_mangling(false)
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    if !Path::new("binaryen/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    gen_bindings();

    if env::var("TARGET").ok().map_or(false, |target| target.contains("emscripten")) {
        let mut build_wasm_binaryen_args = vec![];
        if get_debug() {
            build_wasm_binaryen_args.push("-g");
        }

        let _ = Command::new("./build-binaryen-bc.sh")
            .args(&build_wasm_binaryen_args)
            .status();

        let current_dir = env::current_dir().unwrap();
        println!("cargo:rustc-link-search=native={}", current_dir.to_str().unwrap());
        println!("cargo:rustc-link-lib=static=binaryen-c");
        return;
    }

    let dst = cmake::Config::new("binaryen")
        .define("BUILD_STATIC_LIB", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-lib=static=binaryen");
    println!("cargo:rustc-link-lib=static=asmjs");
    println!("cargo:rustc-link-lib=static=ast");
    println!("cargo:rustc-link-lib=static=cfg");
    println!("cargo:rustc-link-lib=static=passes");
    println!("cargo:rustc-link-lib=static=support");
    println!("cargo:rustc-link-lib=static=wasm");
    println!("cargo:rustc-link-lib=static=emscripten-optimizer");

    // We need to link against C++ std lib
    if let Some(cpp_stdlib) = get_cpp_stdlib() {
        println!("cargo:rustc-link-lib={}", cpp_stdlib);
    }
}

// See https://github.com/alexcrichton/gcc-rs/blob/88ac58e25/src/lib.rs#L1197
fn get_cpp_stdlib() -> Option<String> {
    env::var("TARGET").ok().and_then(|target| {
        if target.contains("msvc") {
            None
        } else if target.contains("darwin") {
            Some("c++".to_string())
        } else if target.contains("freebsd") {
            Some("c++".to_string())
        } else if target.contains("musl") {
            Some("static=stdc++".to_string())
        } else {
            Some("stdc++".to_string())
        }
    })
}

// See https://github.com/alexcrichton/gcc-rs/blob/10871a0e40/src/lib.rs#L1501
fn get_debug() -> bool {
    match env::var("DEBUG").ok() {
        Some(s) => s != "false",
        None => false,
    }
}
