use std::env;
use std::path::PathBuf;

fn main() {
    // ✅ Compile the C runtime
    cc::Build::new()
        .file("src/runtime/wpp_runtime.c")
        .flag_if_supported("-std=c11")
        .cpp(false)
        .compile("wpp_runtime");

    // ✅ Tell Cargo to recompile when the runtime changes
    println!("cargo:rerun-if-changed=src/runtime/wpp_runtime.c");

    // ✅ Tell rustc where to find the compiled static library
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=wpp_runtime");
}
