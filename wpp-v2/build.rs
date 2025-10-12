fn main() {
    cc::Build::new()
        .file("src/runtime/wpp_runtime.c")
        .flag_if_supported("-std=c11")
        .cpp(false) 
        .compile("wpp_runtime");

    println!("cargo:rerun-if-changed=src/runtime/wpp_runtime.c");
}
