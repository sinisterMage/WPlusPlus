mod lexer;
mod parser;
mod ast;
mod macros; // bring wpp_debug! into this binary crate
mod codegen;
pub mod runtime;
pub mod module_system;
pub mod export_resolver;


use std::env;
use std::fs;
use lexer::Lexer;
use parser::Parser;
use codegen::Codegen;
use inkwell::context::Context;

#[link(name = "wpp_runtime", kind = "static")]
unsafe extern "C" {}

fn main() {
    // === Read source file ===
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: wpp-v2 <file.wpp> [--emit-ir]");
        return;
    }

    let path = &args[1];
    let emit_ir = args.iter().any(|a| a == "--emit-ir");

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Failed to read source file {}: {}", path, e);
            return;
        }
    };

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
    
let mut codegen = Codegen::new(&context, "wpp_module", "./src");
let main_fn = codegen.compile_main(&ast);
if let Err(msg) = codegen.module.verify() {
    eprintln!("❌ LLVM Verification failed:\n{}", msg.to_string());
    return;
}
// 🧠 DEBUG: Dump the LLVM IR to console and file
println!("\n🔬 === LLVM IR Dump ===");
codegen.module.print_to_stderr();

codegen.module.print_to_file("debug.ll").unwrap();
println!("💾 IR written to debug.ll");

codegen.run_jit();

}
