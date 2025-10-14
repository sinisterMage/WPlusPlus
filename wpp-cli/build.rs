use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // workspace root (../ from wpp-cli)
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");

    // Try both debug and release build outputs
    let build_modes = ["debug", "release"];
    let mut lib_path: Option<PathBuf> = None;

    for mode in build_modes {
        let build_dir = workspace_root.join(format!("wpp-v2/target/{}/build", mode));
        if let Ok(entries) = fs::read_dir(&build_dir) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("out/libwpp_runtime.a");
                if candidate.exists() {
                    lib_path = Some(candidate);
                    break;
                }
            }
        }
        if lib_path.is_some() {
            break;
        }
    }

    if let Some(lib) = lib_path {
        let out_dir = lib.parent().unwrap();
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=wpp_runtime");
        println!("✅ Linked wpp_runtime from {}", lib.display());
    } else {
        println!("⚠️ Could not locate libwpp_runtime.a — try building wpp-v2 first");
    }

    // Disable Polly optimizations for LLVM-sys
    println!("cargo:rustc-env=LLVM_SYS_NO_POLLY=1");
    println!("cargo:rustc-env=LLVM_SYS_150_NO_POLLY=1");
}
