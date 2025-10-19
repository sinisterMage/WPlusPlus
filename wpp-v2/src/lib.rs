pub mod ast;
pub mod parser;
pub mod codegen;
pub mod lexer;
pub mod runtime;

use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::OptimizationLevel;

use crate::codegen::Codegen;
use crate::lexer::Lexer;
use crate::parser::Parser;
use runtime::*;

/// Link to static runtime (optional)
#[link(name = "wpp_runtime", kind = "static")]
unsafe extern "C" {
    pub fn wpp_print_value_basic(ptr: *const std::ffi::c_void, type_id: i32);
    pub fn wpp_print_array(ptr: *const std::ffi::c_void);
    pub fn wpp_print_object(ptr: *const std::ffi::c_void);
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

    // === Create JIT engine ===
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| format!("JIT init failed: {e:?}"))?;

    unsafe {
        crate::runtime::set_engine(std::mem::transmute::<_, ExecutionEngine<'static>>(engine.clone()));
    }

    // === Register runtime bindings ===
    register_runtime_symbols(&engine, module);

    // === Find and run entrypoint ===
    let entry_name = if module.get_function("bootstrap_main").is_some() {
        "bootstrap_main"
    } else if module.get_function("main").is_some() {
        "main"
    } else {
        "main_async"
    };

    println!("🚀 Launching entrypoint: {entry_name}");

    unsafe {
        let addr = engine
            .get_function_address(entry_name)
            .map_err(|_| format!("❌ No entrypoint function found: {entry_name}"))?;
        let func: extern "C" fn() -> i32 = std::mem::transmute(addr);
        let result = func();
        println!("🏁 Finished running {entry_name}, result = {result}");
    }

    Ok(())
}

/// 🏗️ Compile a W++ source file into LLVM IR (.ll)
pub fn build_ir(source: &str, optimize: bool) -> Result<String, String> {
    let context = Context::create();

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let nodes = parser.parse_program();

    let mut codegen = Codegen::new(&context, "wpp_module");
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
