mod lexer;
mod parser;
mod ast;
mod codegen;

use std::env;
use std::fs;
use lexer::Lexer;
use parser::Parser;
use codegen::Codegen;
use inkwell::context::Context;

fn main() {
    // === Read source file ===
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: wpp-v2 <file.wpp> [--emit-ir]");
        return;
    }

    let path = &args[1];
    let emit_ir = args.iter().any(|a| a == "--emit-ir");

    let source = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read source file: {}", path));

    // === Lexing ===
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("🔤 === Tokens ===");
    for t in &tokens {
        println!("{:?}", t);
    }
    println!("===================");

    // === Parsing ===
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    println!("🔍 === AST Dump ===");
    for node in &ast {
        println!("{:#?}", node);
    }
    println!("===================");

    // === Code generation (LLVM) ===
    let context = Context::create(); // ✅ Create the LLVM context
let mut codegen = Codegen::new(&context, "wpp_module");
let main_fn = codegen.compile_main(&ast);

// 🧠 DEBUG: Dump the LLVM IR to console and file
println!("\n🔬 === LLVM IR Dump ===");
codegen.module.print_to_stderr();

codegen.module.print_to_file("debug.ll").unwrap();
println!("💾 IR written to debug.ll");

codegen.run_jit();

}
