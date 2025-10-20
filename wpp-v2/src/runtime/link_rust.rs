use inkwell::{execution_engine::ExecutionEngine, module::Module};
use libloading::Library;
use crate::module_system::ModuleSystem;

pub fn link_rust_modules<'ctx>(
    engine: &ExecutionEngine<'ctx>,
    module: &Module<'ctx>,
    wms: &ModuleSystem,
) -> Result<(), String> {
    let cache = wms.get_cache();

    for (mod_name, module_data) in cache.iter() {
        if !mod_name.starts_with("rust:") {
            continue;
        }

        let path = &module_data.path;
        if !path.exists() {
            eprintln!("❌ [jit] Rust module file not found: {}", path.display());
            continue;
        }

        println!("🦀 [jit] Linking Rust module '{}'", mod_name);

        unsafe {
            let lib = Library::new(path)
                .map_err(|e| format!("Failed to open Rust lib '{}': {}", path.display(), e))?;

            let exported_symbols = [
                "add", "greet", "init", "deinit", "wpp_plugin_init",
            ];

            for sym_name in &exported_symbols {
                if let Ok(sym) = lib.get::<*const ()>(sym_name.as_bytes()) {
                    if let Some(func_val) = module.get_function(sym_name) {
    unsafe {
        engine.add_global_mapping(&func_val, *sym as usize);
    }
    println!("✅ [jit] Bound Rust fn '{}' from '{}'", sym_name, mod_name);
} else {
    println!("⚠️ [jit] LLVM function '{}' not found — skipping", sym_name);
}

                }
            }

            // keep the library loaded
            std::mem::forget(lib);
        }
    }

    Ok(())
}
