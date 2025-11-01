pub mod ast;
pub mod parser;
pub mod codegen;
pub mod lexer;
pub mod runtime;
pub mod module_system;
pub mod export_resolver;

use std::ffi::CString;

use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::llvm_sys::core::LLVMDeleteGlobal;
use inkwell::llvm_sys::prelude::LLVMValueRef;
use inkwell::llvm_sys::support::LLVMAddSymbol;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{AnyValue, AsValueRef, GlobalValue};
use inkwell::OptimizationLevel;
use libc::{malloc, printf};

use crate::codegen::Codegen;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::runtime::thread::{wpp_mutex_lock, wpp_mutex_new, wpp_mutex_unlock, wpp_thread_join, wpp_thread_join_all, wpp_thread_poll, wpp_thread_spawn_gc, wpp_thread_state_get, wpp_thread_state_new, wpp_thread_state_set};
use runtime::*;

/// Link to static runtime (optional)
#[link(name = "wpp_runtime", kind = "static")]
unsafe extern "C" {
    pub fn wpp_print_value_basic(ptr: *const std::ffi::c_void, type_id: i32);
    pub fn wpp_print_array(ptr: *const std::ffi::c_void);
    pub fn wpp_print_object(ptr: *const std::ffi::c_void);
    pub fn wpp_readline() -> *const std::os::raw::c_char;
    pub fn wpp_int_to_string(value: i32) -> *const std::os::raw::c_char;
}
unsafe fn remove_global(global: &GlobalValue) {
    let raw: LLVMValueRef = global.as_value_ref(); // ✅ correct LLVM type
    LLVMDeleteGlobal(raw); // ✅ correct pointer type
}
unsafe fn add_symbol(name: &str, addr: usize) {
    let cname = CString::new(name).unwrap();
    LLVMAddSymbol(cname.as_ptr(), addr as *mut _);
}
/// Ensure all external runtime functions are declared before JIT creation
fn declare_runtime_externals<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::from(0));

    // === Declare all runtime externals ===
    let externals = [
        // --- Print subsystem ---
        ("wpp_print_value_basic", void_type.fn_type(&[i8_ptr.into(), i32_type.into()], false)),
        ("wpp_print_array", void_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_print_object", void_type.fn_type(&[i8_ptr.into()], false)),

        // --- String subsystem ---
        ("wpp_str_concat", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false)),
        ("wpp_readline", i8_ptr.fn_type(&[], false)),
        ("wpp_int_to_string", i8_ptr.fn_type(&[i32_type.into()], false)),


        // --- HTTP subsystem ---
        ("wpp_http_get", i32_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_http_post", i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false)),
        ("wpp_http_put", i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false)),
        ("wpp_http_patch", i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false)),
        ("wpp_http_delete", i32_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_http_status", i32_type.fn_type(&[i32_type.into()], false)),
        ("wpp_http_body", i8_ptr.fn_type(&[i32_type.into()], false)),
        ("wpp_http_headers", i8_ptr.fn_type(&[i32_type.into()], false)),
        ("wpp_http_free_all", void_type.fn_type(&[], false)),
        ("wpp_register_endpoint", void_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false)),
        ("wpp_start_server", void_type.fn_type(&[i32_type.into()], false)),

        // --- Threading subsystem ---
        ("wpp_thread_spawn_gc", i8_ptr.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_join", void_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_poll", i32_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_state_new", i8_ptr.fn_type(&[i32_type.into()], false)),
        ("wpp_thread_state_get", i8_ptr.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_state_set", void_type.fn_type(&[i8_ptr.into(), i32_type.into()], false)),
        ("wpp_thread_join_all", void_type.fn_type(&[], false)),

        // --- Mutex subsystem ---
        ("wpp_mutex_new", i8_ptr.fn_type(&[i32_type.into()], false)),
        ("wpp_mutex_lock", void_type.fn_type(&[i8_ptr.into(), i32_type.into()], false)),
        ("wpp_mutex_unlock", void_type.fn_type(&[i8_ptr.into()], false)),

        // --- Runtime ---
        ("wpp_runtime_wait", void_type.fn_type(&[], false)),
        ("wpp_return", void_type.fn_type(&[i8_ptr.into(), i32_type.into()], false)),

        // --- libc ---
        ("printf", i32_type.fn_type(&[i8_ptr.into()], true)),
        ("malloc", i8_ptr.fn_type(&[i64_type.into()], false)),
    ];

    for (name, ty) in externals {
        if module.get_function(name).is_none() {
            module.add_function(name, ty, None);
        }
    }
}
fn register_all_runtime_symbols() {
    unsafe {
        // --- Print subsystem ---
        add_symbol("wpp_print_value_basic", wpp_print_value_basic as usize);
        add_symbol("wpp_print_array", wpp_print_array as usize);
        add_symbol("wpp_print_object", wpp_print_object as usize);

        // --- String subsystem ---
        add_symbol("wpp_str_concat", wpp_str_concat as usize);
        add_symbol("wpp_readline", wpp_readline as usize);
        add_symbol("wpp_int_to_string", wpp_int_to_string as usize);

        // --- HTTP subsystem ---
        add_symbol("wpp_http_get", wpp_http_get as usize);
        add_symbol("wpp_http_post", wpp_http_post as usize);
        add_symbol("wpp_http_put", wpp_http_put as usize);
        add_symbol("wpp_http_patch", wpp_http_patch as usize);
        add_symbol("wpp_http_delete", wpp_http_delete as usize);
        add_symbol("wpp_http_status", wpp_http_status as usize);
        add_symbol("wpp_http_body", wpp_http_body as usize);
        add_symbol("wpp_http_headers", wpp_http_headers as usize);
        add_symbol("wpp_http_free_all", wpp_http_free_all as usize);
        add_symbol("wpp_register_endpoint", wpp_register_endpoint as usize);
        add_symbol("wpp_start_server", wpp_start_server as usize);

        // --- Threading subsystem ---
        add_symbol("wpp_thread_spawn_gc", wpp_thread_spawn_gc as usize);
        add_symbol("wpp_thread_join", wpp_thread_join as usize);
        add_symbol("wpp_thread_poll", wpp_thread_poll as usize);
        add_symbol("wpp_thread_state_new", wpp_thread_state_new as usize);
        add_symbol("wpp_thread_state_get", wpp_thread_state_get as usize);
        add_symbol("wpp_thread_state_set", wpp_thread_state_set as usize);
        add_symbol("wpp_thread_join_all", wpp_thread_join_all as usize);

        // --- Mutex subsystem ---
        add_symbol("wpp_mutex_new", wpp_mutex_new as usize);
        add_symbol("wpp_mutex_lock", wpp_mutex_lock as usize);
        add_symbol("wpp_mutex_unlock", wpp_mutex_unlock as usize);

        // --- Runtime ---
        add_symbol("wpp_runtime_wait", wpp_runtime_wait as usize);

        // --- libc ---
        unsafe extern "C" {
            fn printf(fmt: *const std::os::raw::c_char, ...) -> i32;
            fn malloc(size: usize) -> *mut std::ffi::c_void;
        }
        add_symbol("printf", printf as usize);
        add_symbol("malloc", malloc as usize);
    }
}

fn register_runtime_symbols<'ctx>(engine: &ExecutionEngine<'ctx>, module: &Module<'ctx>) {
    let map_fn = |name: &str, ptr: usize| {
        if let Some(func) = module.get_function(name) {
            engine.add_global_mapping(&func, ptr);
            #[cfg(debug_assertions)]
            println!("🔗 Bound {name}");
        } else {
            #[cfg(debug_assertions)]
            eprintln!("⚠️ Function {name} not found in module");
        }
    };

    unsafe {
        // === Printing subsystem ===
        map_fn("wpp_print_value_basic", wpp_print_value_basic as usize);
        map_fn("wpp_print_array", wpp_print_array as usize);
        map_fn("wpp_print_object", wpp_print_object as usize);
        map_fn("wpp_readline", wpp_readline as usize);

        // === String subsystem ===
        map_fn("wpp_str_concat", wpp_str_concat as usize);
        map_fn("wpp_int_to_string", wpp_int_to_string as usize);

        // === HTTP subsystem ===
        map_fn("wpp_http_get", wpp_http_get as usize);
        map_fn("wpp_http_post", wpp_http_post as usize);
        map_fn("wpp_http_put", wpp_http_put as usize);
        map_fn("wpp_http_patch", wpp_http_patch as usize);
        map_fn("wpp_http_delete", wpp_http_delete as usize);
        map_fn("wpp_http_status", wpp_http_status as usize);
        map_fn("wpp_http_body", wpp_http_body as usize);
        map_fn("wpp_http_headers", wpp_http_headers as usize);
        map_fn("wpp_http_free_all", wpp_http_free_all as usize);
        map_fn("wpp_register_endpoint", wpp_register_endpoint as usize);
        map_fn("wpp_start_server", wpp_start_server as usize);

        // === Threading subsystem ===
        map_fn("wpp_thread_spawn_gc", wpp_thread_spawn_gc as usize);
        map_fn("wpp_thread_join", wpp_thread_join as usize);
        map_fn("wpp_thread_poll", wpp_thread_poll as usize);
        map_fn("wpp_thread_state_new", wpp_thread_state_new as usize);
        map_fn("wpp_thread_state_get", wpp_thread_state_get as usize);
        map_fn("wpp_thread_state_set", wpp_thread_state_set as usize);
        map_fn("wpp_thread_join_all", wpp_thread_join_all as usize);

        // === Mutex subsystem ===
        map_fn("wpp_mutex_new", wpp_mutex_new as usize);
        map_fn("wpp_mutex_lock", wpp_mutex_lock as usize);
        map_fn("wpp_mutex_unlock", wpp_mutex_unlock as usize);

        // === Runtime ===
        map_fn("wpp_runtime_wait", wpp_runtime_wait as usize);
        map_fn("wpp_return", wpp_return as usize);

        // === Standard libc ===
        unsafe extern "C" {
            fn printf(fmt: *const std::os::raw::c_char, ...) -> i32;
            fn malloc(size: usize) -> *mut std::ffi::c_void;
        }
        map_fn("printf", printf as usize);
        map_fn("malloc", malloc as usize);
    }
}


/// 🦥 Run a W++ source file using the LLVM JIT engine
pub fn run_file(codegen: &mut Codegen, optimize: bool) -> Result<(), String> {
    use std::mem;
    use inkwell::passes::PassManager;
    use inkwell::OptimizationLevel;
    use inkwell::memory_buffer::MemoryBuffer;

    let module = &codegen.module;
    let context = codegen.context;

    // === Ensure libc symbols exist ===
    let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::from(0));
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();

    if module.get_function("printf").is_none() {
        module.add_function("printf", i32_type.fn_type(&[i8_ptr.into()], true), None);
    }
    if module.get_function("malloc").is_none() {
        module.add_function("malloc", i8_ptr.fn_type(&[i64_type.into()], false), None);
    }
    
    // === Apply optimizations (optional) ===
    if optimize {
        let pm = PassManager::create(());
        pm.add_instruction_combining_pass();
        pm.add_reassociate_pass();
        pm.add_gvn_pass();
        pm.add_cfg_simplification_pass();
        pm.add_promote_memory_to_register_pass();
        pm.run_on(module);
        println!("✨ Applied LLVM optimization passes.");
    }

    // === Merge submodules into main LLVM module ===
    if let Some(wms_arc) = &codegen.wms {
        println!("🧩 Linking all submodules into main LLVM module...");

        // === Step 1: Reload main from WMS cache safely ===
        {
            let wms = wms_arc.lock().unwrap();

            if module.get_function("main").is_none() && module.get_function("main_async").is_none() {
                if let Some(main_dep) = wms.get_cache().get("main") {
                    if let Some(ref ir_text) = main_dep.llvm_ir {
                        println!("🧩 Reloading main module from cache into JIT context...");

                        let ir_bytes = ir_text.as_bytes().to_vec();
                        let mem_buf = unsafe {
                            MemoryBuffer::create_from_memory_range_copy(&ir_bytes, "main_clone")
                        };

                        if let Ok(main_mod) = codegen.context.create_module_from_ir(mem_buf) {
                            // 🧹 Remove duplicate globals
                            for global_name in ["_wpp_exc_flag", "_wpp_exc_i32", "_wpp_exc_str"] {
                                if let Some(global) = main_mod.get_global(global_name) {
                                    #[cfg(debug_assertions)]
                                    println!(
                                        "🔧 [wms] Neutralized duplicate global '{}' in reloaded main module",
                                        global_name
                                    );
                                    unsafe { remove_global(&global); }
                                }
                            }

                            // 🧹 Also remove duplicate main_async before link
                            if let Some(existing) = codegen.module.get_function("main_async") {
                                println!("🧹 Removing old main_async before reattaching cached main");
                                unsafe { existing.delete() };
                            }

                            // Now safely link the main module
                            if let Err(e) = codegen.module.link_in_module(main_mod) {
                                eprintln!("❌ Failed to link cached main module: {e:?}");
                            } else {
                                println!("✅ Successfully reattached main module before linking submodules.");
                            }
                        }
                    }
                }
            }
        }

        // === Step 2: Lock once for normal submodule linking ===
        let wms = wms_arc.lock().unwrap();
        let cache_guard = wms.get_cache();

        for (name, dep) in cache_guard.iter() {
            // 🛑 Skip linking the main module itself
            if name == "main" || name == "src/main" || name.ends_with("/main") {
                println!("⚙️ Skipping self-link of main module ({name})");
                continue;
            }

            if let Some(ref ir_text) = dep.llvm_ir {
                #[cfg(debug_assertions)]
                println!("🔗 [link] Importing compiled submodule: {}", name);
                let mem_buf = MemoryBuffer::create_from_memory_range_copy(ir_text.as_bytes(), name);

                match codegen.context.create_module_from_ir(mem_buf) {
                    Ok(submod) => {
                        if submod.get_first_function().is_none() {
                            println!("⚠️ Skipping empty submodule '{}'", name);
                            continue;
                        }
                        if let Err(e) = codegen.module.link_in_module(submod) {
                            eprintln!("❌ Failed to link submodule {name}: {e:?}");
                        } else {
                            println!("✅ Successfully linked submodule: {}", name);
                        }
                    }
                    Err(e) => eprintln!("❌ Failed to parse IR for submodule {name}: {e:?}"),
                }
            } else {
                eprintln!("⚠️ Submodule '{}' has no LLVM IR; did it compile successfully?", name);
            }
        }
    }
    #[cfg(debug_assertions)]
    println!("🪶 [debug1] Finished merging and linking modules — about to declare externals");
    declare_runtime_externals(context, module);
    #[cfg(debug_assertions)]
    println!("🪶 [debug2] Declared externals successfully — creating JIT engine next");
    register_all_runtime_symbols();


    // === Create JIT engine ===
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| format!("JIT init failed: {e:?}"))?;
    // === Optional: Link native Rust modules (.so/.dylib) ===
{
    println!("🧩 Linking Rust modules into JIT context...");
    if let Some(wms_arc) = &codegen.wms {
        let wms = wms_arc.lock().unwrap();
        if let Err(e) = crate::runtime::link_rust_modules(&engine, module, &wms) {
            eprintln!("⚠️ [jit] Failed to link Rust modules: {e}");
        } else {
            println!("✅ [jit] Rust modules linked successfully.");
        }
    }
}

    #[cfg(debug_assertions)]
println!("🪶 [debug3] JIT engine created successfully");
    // TEMPORARILY COMMENTED OUT FOR DEBUGGING
    // for func in module.get_functions() {
    // if let Ok(name) = func.get_name().to_str() {
    //     if let Ok(addr) = engine.get_function_address(name) {
    //         #[cfg(debug_assertions)]
    //         println!("🔎 {name} -> 0x{addr:x}");
    //     } else {
    //         #[cfg(debug_assertions)]
    //         println!("❌ {name} -> <missing>");
    //     }
    // }
    // }

    println!("🪶 [debug3.1] About to set engine globally");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    unsafe {
        println!("🪶 [debug3.1a] Inside unsafe block");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        crate::runtime::set_engine(std::mem::transmute::<_, ExecutionEngine<'static>>(engine.clone()));
        println!("🪶 [debug3.1b] set_engine returned");
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
    println!("🪶 [debug3.2] Engine set, now registering runtime symbols");
    std::io::Write::flush(&mut std::io::stdout()).ok();

    // === Register runtime bindings ===
    register_runtime_symbols(&engine, module);
    #[cfg(debug_assertions)]
    println!("🪶 [debug4] Runtime symbols bound successfully");

    // === Link cross-module imports (from WMS) ===
    if let (Some(wms_arc), Some(resolver_arc)) = (&codegen.wms, &codegen.resolver) {
        println!("🧩 Linking runtime imports across modules...");

        let wms = wms_arc.lock().unwrap();
        let mut resolver = resolver_arc.lock().unwrap();
        resolver.link_imports_runtime(&engine, module, &wms);
        #[cfg(debug_assertions)]
        println!("🪶 [debug5] Finished linking runtime imports across modules");

    }
    #[cfg(debug_assertions)]
    println!("🪶 [debug6] Searching for entrypoint (bootstrap_main / main / main_async)");
    // === Find and run entrypoint ===
    let entry_name = if module.get_function("bootstrap_main").is_some() {
        "bootstrap_main"
    } else if module.get_function("main").is_some() {
        "main"
    } else if module.get_function("main_async").is_some() {
        "main_async"
    } else {
        eprintln!("❌ No valid entrypoint found (expected main, main_async, or bootstrap_main)");
        return Err("❌ No entrypoint function found in final linked module".into());
    };
    #[cfg(debug_assertions)]
    println!("🪶 [debug7] Entrypoint resolved to {entry_name}");
    #[cfg(debug_assertions)]
    println!("🔍 [jit] Listing all LLVM functions before execution:");
    // TEMPORARILY COMMENTED - causes segfault
    // for func in module.get_functions() {
    //     if let Ok(name) = func.get_name().to_str() {
    //         #[cfg(debug_assertions)]
    //         println!("  - {}", name);
    //     }
    // }

    if let Some(wms_arc) = &codegen.wms {
        let wms = wms_arc.lock().unwrap();
        println!("🧩 WMS available with {} cached modules", wms.get_cache().len());
    } else {
        println!("❌ No WMS attached to codegen at runtime!");
    }

    // === Debug entrypoint signature ===
    println!("🚀 Launching entrypoint: {entry_name}");
    if let Some(func) = module.get_function(entry_name) {
        #[cfg(debug_assertions)]
        println!("🔍 LLVM signature for entrypoint:");
        #[cfg(debug_assertions)]
        println!("{}", func.print_to_string().to_string());
        #[cfg(debug_assertions)]
        println!("=== FULL IR DUMP ===");
        #[cfg(debug_assertions)]
println!("{}", module.print_to_string().to_string());
    }

    // === Verify runtime bindings before executing ===
    // TEMPORARILY COMMENTED - causes segfault
    // for name in ["wpp_print_value_basic", "wpp_thread_join_all", "printf", "malloc"] {
    //     match engine.get_function_address(name) {
    //         Ok(addr) => println!("✅ Verified binding for {name} @ 0x{addr:x}"),
    //         Err(_) => println!("❌ Missing runtime binding for {name}"),
    //     }
    // }
    #[cfg(debug_assertions)]
    println!("🧠 [jit] About to execute entrypoint `{entry_name}`");
    // === Run the entrypoint ===
    
    unsafe {
        let addr = engine
            .get_function_address(entry_name)
            .map_err(|_| format!("❌ No entrypoint function found: {entry_name}"))?;
        let func: extern "C" fn() -> i32 = std::mem::transmute(addr);
        let result = func();
        println!("✅ [jit] Returned cleanly from `{entry_name}` with result = {result}");
        println!("🏁 Finished running {entry_name}, result = {result}");
    }
    #[cfg(debug_assertions)]
    println!("🪶 [debug10] run_file() completed successfully without panic");
    Ok(())
}


/// 🏗️ Compile a W++ source file into LLVM IR (.ll)
pub fn build_ir(source: &str, optimize: bool) -> Result<String, String> {
    let context = Context::create();

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let nodes = parser.parse_program();

    let mut codegen = Codegen::new(&context, "wpp_module", "./src");
    codegen.compile_main(&nodes);

    if optimize {
        let pass_manager = PassManager::create(());
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        pass_manager.add_gvn_pass();
        pass_manager.add_cfg_simplification_pass();
        pass_manager.add_basic_alias_analysis_pass();
        pass_manager.add_promote_memory_to_register_pass();
        pass_manager.run_on(&codegen.module);
        println!("✨ Applied LLVM optimization passes.");
    }

    Ok(codegen.module.print_to_string().to_string())
}
