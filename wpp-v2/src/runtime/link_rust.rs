use inkwell::{execution_engine::ExecutionEngine, module::Module, AddressSpace};
use libloading::Library;
use crate::module_system::ModuleSystem;

// List of known Rust FFI function names to auto-declare
const KNOWN_FFI_FUNCTIONS: &[(&str, &str)] = &[
    // JSON library functions
    ("json_parse", "ptr->ptr"),
    ("json_stringify", "ptr->ptr"),
    ("json_pretty", "ptr_i32->ptr"),
    ("json_validate", "ptr->i32"),
    ("json_get", "ptr_ptr->ptr"),
    ("json_get_string", "ptr_ptr->ptr"),
    ("json_get_int", "ptr_ptr->i32"),
    ("json_merge", "ptr_ptr->ptr"),
    ("json_free", "ptr->void"),

    // File I/O library functions
    ("io_read_file", "ptr->ptr"),
    ("io_write_file", "ptr_ptr->i32"),
    ("io_read_bytes", "ptr->ptr"),
    ("io_read_lines", "ptr->ptr"),
    ("io_append_file", "ptr_ptr->i32"),
    ("io_write_bytes", "ptr_ptr->i32"),
    ("io_exists", "ptr->i32"),
    ("io_delete_file", "ptr->i32"),
    ("io_copy_file", "ptr_ptr->i32"),
    ("io_rename_file", "ptr_ptr->i32"),
    ("io_file_size", "ptr->i64"),
    ("io_is_file", "ptr->i32"),
    ("io_is_dir", "ptr->i32"),
    ("io_create_dir", "ptr->i32"),
    ("io_create_dir_all", "ptr->i32"),
    ("io_remove_dir", "ptr->i32"),
    ("io_remove_dir_all", "ptr->i32"),
    ("io_list_dir", "ptr->ptr"),
    ("io_free", "ptr->void"),

    // CORS library functions
    ("cors_strlen", "ptr->i32"),
    ("cors_int_to_string", "i32->ptr"),
    ("cors_contains", "ptr_ptr->i32"),
    ("cors_strcmp", "ptr_ptr->i32"),
    ("cors_strcasecmp", "ptr_ptr->i32"),
    ("cors_is_origin_allowed", "ptr_ptr->i32"),
    ("cors_is_method_allowed", "ptr_ptr->i32"),
    ("cors_are_headers_allowed", "ptr_ptr->i32"),
    ("cors_is_preflight", "ptr->i32"),
    ("cors_gc_collect", "->i32"),
    ("cors_gc_shutdown", "->void"),

    // MySQL database driver functions
    ("mysql_connect", "ptr_ptr_ptr_ptr->ptr"),
    ("mysql_close", "ptr->i32"),
    ("mysql_query", "ptr_ptr->ptr"),
    ("mysql_execute", "ptr_ptr->i64"),
    ("mysql_prepare", "ptr_ptr->ptr"),
    ("mysql_bind_execute", "ptr_ptr->ptr"),
    ("mysql_begin_transaction", "ptr->i32"),
    ("mysql_commit", "ptr->i32"),
    ("mysql_rollback", "ptr->i32"),
    ("mysql_get_last_error", "->ptr"),
    ("mysql_gc_collect", "->i32"),
    ("mysql_gc_shutdown", "->void"),

    // PostgreSQL database driver functions
    ("pg_connect", "ptr->ptr"),
    ("pg_close", "ptr->i32"),
    ("pg_query", "ptr_ptr->ptr"),
    ("pg_execute", "ptr_ptr->i64"),
    ("pg_begin_transaction", "ptr->i32"),
    ("pg_commit", "ptr->i32"),
    ("pg_rollback", "ptr->i32"),
    ("pg_get_last_error", "->ptr"),
    ("pg_gc_collect", "->i32"),
    ("pg_gc_shutdown", "->void"),

    // MongoDB database driver functions
    ("mongo_connect", "ptr->ptr"),
    ("mongo_close", "ptr->i32"),
    ("mongo_find", "ptr_ptr_ptr_ptr->ptr"),
    ("mongo_insert_one", "ptr_ptr_ptr_ptr->ptr"),
    ("mongo_update_one", "ptr_ptr_ptr_ptr_ptr->i64"),
    ("mongo_delete_one", "ptr_ptr_ptr_ptr->i64"),
    ("mongo_get_last_error", "->ptr"),
    ("mongo_gc_collect", "->i32"),
    ("mongo_gc_shutdown", "->void"),

    // Firebase database driver functions
    ("firebase_init", "ptr_ptr->ptr"),
    ("firebase_close", "ptr->i32"),
    ("firebase_set", "ptr_ptr_ptr->i32"),
    ("firebase_get", "ptr_ptr->ptr"),
    ("firebase_delete", "ptr_ptr->i32"),
    ("firebase_get_last_error", "->ptr"),
    ("firebase_gc_collect", "->i32"),
    ("firebase_gc_shutdown", "->void"),

    // Apache Cassandra database driver functions
    ("cassandra_connect", "ptr_ptr->ptr"),
    ("cassandra_close", "ptr->i32"),
    ("cassandra_query", "ptr_ptr->ptr"),
    ("cassandra_execute", "ptr_ptr->i32"),
    ("cassandra_get_last_error", "->ptr"),
    ("cassandra_gc_collect", "->i32"),
    ("cassandra_gc_shutdown", "->void"),
];

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

            // STEP 1: Declare known FFI functions in LLVM IR if they don't exist
            let mut declared_count = 0;
            for (func_name, signature) in KNOWN_FFI_FUNCTIONS {
                // Check if symbol exists in the loaded library
                if lib.get::<*const ()>(func_name.as_bytes()).is_ok() {
                    // Only declare if not already in module
                    if module.get_function(func_name).is_none() {
                        declare_ffi_function(module, func_name, signature);
                        declared_count += 1;
                        println!("📝 [jit] Declared FFI function '{}' ({})", func_name, signature);
                    }
                }
            }

            if declared_count > 0 {
                println!("📝 [jit] Declared {} FFI functions for '{}'", declared_count, mod_name);
            }

            // STEP 2: Dynamic symbol binding - bind all declared functions to library symbols
            let mut bound_count = 0;
            let mut attempted_count = 0;

            // Iterate through all functions in the LLVM module
            let mut current_func = module.get_first_function();
            while let Some(func_val) = current_func {
                if let Ok(func_name) = func_val.get_name().to_str() {
                    // Skip W++ runtime functions and LLVM intrinsics
                    if !func_name.starts_with("wpp_") &&
                       !func_name.starts_with("llvm.") &&
                       !func_name.contains("__") {  // Skip mangled W++ functions

                        attempted_count += 1;

                        // Try to get symbol from Rust library
                        if let Ok(sym) = lib.get::<*const ()>(func_name.as_bytes()) {
                            engine.add_global_mapping(&func_val, *sym as usize);
                            println!("✅ [jit] Bound Rust fn '{}' from '{}'", func_name, mod_name);
                            bound_count += 1;
                        }
                    }
                }
                current_func = func_val.get_next_function();
            }

            if bound_count > 0 {
                println!("🦀 [jit] Successfully bound {}/{} functions from '{}'",
                         bound_count, attempted_count, mod_name);
            } else if declared_count == 0 {
                println!("⚠️ [jit] No symbols bound from '{}' (tried {} functions)",
                         mod_name, attempted_count);
            }

            // keep the library loaded
            std::mem::forget(lib);
        }
    }

    Ok(())
}

// Helper function to declare FFI functions with specific signatures
fn declare_ffi_function(
    module: &Module,
    name: &str,
    signature: &str,
) {
    let context = module.get_context();
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = context.i32_type();
    let i64_ty = context.i64_type();
    let void_ty = context.void_type();

    let fn_type = match signature {
        // String -> String functions
        "ptr->ptr" => i8_ptr.fn_type(&[i8_ptr.into()], false),

        // String + Int -> String functions
        "ptr_i32->ptr" => i8_ptr.fn_type(&[i8_ptr.into(), i32_ty.into()], false),

        // String -> Int functions
        "ptr->i32" => i32_ty.fn_type(&[i8_ptr.into()], false),

        // String -> Long functions
        "ptr->i64" => i64_ty.fn_type(&[i8_ptr.into()], false),

        // Two Strings -> String functions
        "ptr_ptr->ptr" => i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),

        // Two Strings -> Int functions
        "ptr_ptr->i32" => i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false),

        // String -> Void functions
        "ptr->void" => void_ty.fn_type(&[i8_ptr.into()], false),

        _ => {
            eprintln!("⚠️ Unknown FFI signature: {}", signature);
            return;
        }
    };

    module.add_function(name, fn_type, None);
}
