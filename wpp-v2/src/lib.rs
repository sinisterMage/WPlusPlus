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
use crate::runtime::thread::{wpp_mutex_lock, wpp_mutex_new, wpp_mutex_unlock, wpp_thread_join, wpp_thread_join_all, wpp_thread_poll, wpp_thread_spawn_gc};
use runtime::*;

/// Link to static runtime (optional)
#[link(name = "wpp_runtime", kind = "static")]
unsafe extern "C" {
    pub fn wpp_print_value_basic(ptr: *const std::ffi::c_void, type_id: i32);
    pub fn wpp_print_array(ptr: *const std::ffi::c_void);
    pub fn wpp_print_object(ptr: *const std::ffi::c_void);
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

    // === Declare all known externals ===
    let externals = [
        ("wpp_print_value_basic", void_type.fn_type(&[i8_ptr.into(), i32_type.into()], false)),
        ("wpp_print_array", void_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_print_object", void_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_spawn_gc", i8_ptr.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_join", void_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_poll", i32_type.fn_type(&[i8_ptr.into()], false)),
        ("wpp_thread_join_all", void_type.fn_type(&[], false)), // 🧩 critical
        ("wpp_mutex_new", i8_ptr.fn_type(&[i32_type.into()], false)),
        ("wpp_mutex_lock", void_type.fn_type(&[i8_ptr.into(), i32_type.into()], false)),
        ("wpp_mutex_unlock", void_type.fn_type(&[i8_ptr.into()], false)),
        ("printf", i32_type.fn_type(&[i8_ptr.into()], true)),
        ("malloc", i8_ptr.fn_type(&[i64_type.into()], false)),
    ];

    for (name, ty) in externals {
        if module.get_function(name).is_none() {
            module.add_function(name, ty, None);
            println!("🔧 Declared external runtime function: {name}");
        }
    }
}

fn register_runtime_symbols<'ctx>(engine: &ExecutionEngine<'ctx>, module: &Module<'ctx>) {
    let map_fn = |name: &str, ptr: usize| {
        if let Some(func) = module.get_function(name) {
            engine.add_global_mapping(&func, ptr);
            println!("🔗 Bound {name}");
        } else {
            eprintln!("⚠️ Function {name} not found in module");
        }
    };

    unsafe {
        // === Printing subsystem ===
        map_fn("wpp_print_value_basic", wpp_print_value_basic as usize);
        map_fn("wpp_print_array", wpp_print_array as usize);
        map_fn("wpp_print_object", wpp_print_object as usize);

        // === Threading subsystem ===
        map_fn("wpp_thread_spawn_gc", wpp_thread_spawn_gc as usize);
        map_fn("wpp_thread_join", wpp_thread_join as usize);
        map_fn("wpp_thread_poll", wpp_thread_poll as usize);
        map_fn("wpp_thread_join_all", wpp_thread_join_all as usize);
        map_fn("wpp_mutex_new", wpp_mutex_new as usize);
        map_fn("wpp_mutex_lock", wpp_mutex_lock as usize);
        map_fn("wpp_mutex_unlock", wpp_mutex_unlock as usize);

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
    println!("🪶 [debug1] Finished merging and linking modules — about to declare externals");
    declare_runtime_externals(context, module);
    println!("🪶 [debug2] Declared externals successfully — creating JIT engine next");
    unsafe {
    add_symbol("wpp_print_value_basic", wpp_print_value_basic as usize);
    add_symbol("wpp_thread_join_all", wpp_thread_join_all as usize);
    add_symbol("printf", printf as usize);
    add_symbol("malloc", malloc as usize);
}


    // === Create JIT engine ===
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| format!("JIT init failed: {e:?}"))?;
println!("🪶 [debug3] JIT engine created successfully");
    for func in module.get_functions() {
    if let Ok(name) = func.get_name().to_str() {
        if let Ok(addr) = engine.get_function_address(name) {
            println!("🔎 {name} -> 0x{addr:x}");
        } else {
            println!("❌ {name} -> <missing>");
        }
    }
}

    unsafe {
        crate::runtime::set_engine(std::mem::transmute::<_, ExecutionEngine<'static>>(engine.clone()));
    }

    // === Register runtime bindings ===
    register_runtime_symbols(&engine, module);
    println!("🪶 [debug4] Runtime symbols bound successfully");

    // === Link cross-module imports (from WMS) ===
    if let (Some(wms_arc), Some(resolver_arc)) = (&codegen.wms, &codegen.resolver) {
        println!("🧩 Linking runtime imports across modules...");

        let wms = wms_arc.lock().unwrap();
        let mut resolver = resolver_arc.lock().unwrap();
        resolver.link_imports_runtime(&engine, module, &wms);
        println!("🪶 [debug5] Finished linking runtime imports across modules");

    }
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
    println!("🪶 [debug7] Entrypoint resolved to {entry_name}");
    println!("🔍 [jit] Listing all LLVM functions before execution:");
    for func in module.get_functions() {
        if let Ok(name) = func.get_name().to_str() {
            println!("  - {}", name);
        }
    }

    if let Some(wms_arc) = &codegen.wms {
        let wms = wms_arc.lock().unwrap();
        println!("🧩 WMS available with {} cached modules", wms.get_cache().len());
    } else {
        println!("❌ No WMS attached to codegen at runtime!");
    }

    // === Debug entrypoint signature ===
    println!("🚀 Launching entrypoint: {entry_name}");
    if let Some(func) = module.get_function(entry_name) {
        println!("🔍 LLVM signature for entrypoint:");
        println!("{}", func.print_to_string().to_string());
        println!("=== FULL IR DUMP ===");
println!("{}", module.print_to_string().to_string());
    }

    // === Verify runtime bindings before executing ===
    for name in ["wpp_print_value_basic", "wpp_thread_join_all", "printf", "malloc"] {
        match engine.get_function_address(name) {
            Ok(addr) => println!("✅ Verified binding for {name} @ 0x{addr:x}"),
            Err(_) => println!("❌ Missing runtime binding for {name}"),
        }
    }
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
