mod ast;
mod parser;
mod codegen;

use ast::*;
use parser::Parser;
use codegen::Codegen;
use inkwell::context::Context;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: wpp-llvm <file.wpp>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let source =
        fs::read_to_string(filename).unwrap_or_else(|_| panic!("Failed to read {}", filename));

    // Parse the source
    let parser = Parser::new(&source);
    let nodes = parser.parse();

    // Generate LLVM
    let context = Context::create();
    let mut cg = Codegen::new(&context, filename);
    cg.compile_main(&nodes);
    cg.module.print_to_stderr();
    cg.run_jit();
}
