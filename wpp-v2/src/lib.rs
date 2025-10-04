pub mod ast;
pub mod parser;
pub mod codegen;
pub mod lexer;
pub mod runtime;

use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::passes::PassManager;
use inkwell::OptimizationLevel;
use crate::codegen::Codegen;
use crate::lexer::Lexer;
use crate::parser::Parser;

/// 🦥 Run a W++ source file using the LLVM JIT engine
pub fn run_file(source: &str, optimize: bool) -> Result<(), String> {
    // === Step 1: Create LLVM context ===
    let context = Context::create();

    // === Step 2: Tokenize & parse ===
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let nodes = parser.parse_program();

    // === Step 3: Generate LLVM IR ===
    let mut codegen = Codegen::new(&context, "wpp_main");
    codegen.compile_main(&nodes);
    let module = &codegen.module;

    // === Step 4: Optionally apply optimization passes ===
    if optimize {
        // ✅ Create a module-level pass manager
        let pass_manager = PassManager::create(());

        // Add optimization passes
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        pass_manager.add_gvn_pass();
        pass_manager.add_cfg_simplification_pass();
        pass_manager.add_basic_alias_analysis_pass();
        pass_manager.add_promote_memory_to_register_pass();

        // ✅ Run on the entire module
        pass_manager.run_on(module);
        println!("✨ Applied LLVM optimization passes.");
    }

    // === Step 5: Create JIT execution engine ===
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| format!("JIT init failed: {e:?}"))?;

    // === Step 6: Find and execute main() ===
    unsafe {
        let func_addr = execution_engine
            .get_function_address("main")
            .map_err(|_| "❌ No main() function found")?;

        let main: extern "C" fn() = std::mem::transmute(func_addr);
        main();
    }

    Ok(())
}

/// 🏗️ Compile a W++ source file into LLVM IR (.ll)
pub fn build_ir(source: &str, optimize: bool) -> Result<String, String> {
    // === Step 1: Create LLVM context ===
    let context = Context::create();

    // === Step 2: Tokenize & parse ===
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let nodes = parser.parse_program();

    // === Step 3: Compile to LLVM IR ===
    let mut codegen = Codegen::new(&context, "wpp_module");
    codegen.compile_main(&nodes);

    // === Step 4: Optionally apply optimization passes ===
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

    // === Step 5: Return IR as string ===
    Ok(codegen.module.print_to_string().to_string())
}
