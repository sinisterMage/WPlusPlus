//! W++ Export Resolver
//! Links exported functions/consts between modules after parsing and during runtime.

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::{Linkage, Module};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::FunctionValue;

use crate::ast::{Expr, Node};
use crate::module_system::ModuleSystem;
use std::collections::HashMap;

/// Represents a global namespace of symbols (function names, constants, etc.)
pub struct ExportResolver {
    /// Global table: symbol name → exported node (function, const, etc.)
    pub global_table: HashMap<String, Node>,
}

impl ExportResolver {
    pub fn new() -> Self {
        Self {
            global_table: HashMap::new(),
        }
    }

    /// 🧠 Collect all exported items from all modules in WMS cache.
    /// Builds a global symbol table for later use.
    pub fn collect_exports(&mut self, wms: &ModuleSystem) {
        let cache = wms.get_cache();

        for (name, module) in cache.iter() {
            for node in &module.ast {
                if let Node::Export { name: export_name, item } = node {
                    // Store the exported function/const node
                    self.global_table.insert(export_name.clone(), *item.clone());
                    println!("📦 [resolver] Registered export '{}::{}'", name, export_name);
                }
            }
        }
    }

    /// 🧩 Inject all imported items directly into a module AST.
    /// Ensures imports exist during codegen (no undefined function calls).
    pub fn inject_imports(&self, ast: &mut Vec<Node>) -> Result<(), String> {
        let mut injected = Vec::new();

        for node in ast.iter() {
            match node {
                Node::ImportList { module: _, members } => {
                    for (name, alias) in members {
                        let key = name.clone();
                        if let Some(exported) = self.global_table.get(&key) {
                            let injected_name = alias.clone().unwrap_or_else(|| key.clone());
                            injected.push(Node::Let {
                                name: injected_name,
                                value: match exported {
                                    Node::Expr(e) => e.clone(),
                                    Node::Export { item, .. } => match &**item {
                                        Node::Expr(inner) => inner.clone(),
                                        _ => return Err(format!("Unsupported export item: {key}")),
                                    },
                                    _ => return Err(format!("Unsupported export node: {key}")),
                                },
                                is_const: true,
                                ty: None,
                            });
                        } else {
                            return Err(format!("Unknown imported symbol: {}", key));
                        }
                    }
                }
                _ => {}
            }
        }

        // Prepend injected imports
        injected.append(ast);
        *ast = injected;
        Ok(())
    }

    /// 🧱 At IR level: add external stubs for all imported functions.
    /// This ensures LLVM sees all called functions as valid externals.
    pub fn apply_imports<'ctx>(&self, llvm_module: &mut Module<'ctx>, wms: &ModuleSystem) {
        let cache = wms.get_cache();

        for (_module_name, module_data) in cache.iter() {
            for node in &module_data.ast {
                match node {
                    Node::ImportList { module, members } => {
                        println!("🔗 [resolver] Linking selective imports from '{}'", module);

                        for (member_name, _alias_opt) in members {
                            if let Some(exported) = self.global_table.get(member_name) {
                                // Only process function exports
                                if let Node::Expr(Expr::Funcy { name, params, .. }) = exported {
                                    let ctx = llvm_module.get_context();

                                    let ret_ty = ctx.void_type(); // assume void for now
                                    let param_tys: Vec<BasicMetadataTypeEnum> = params
                                        .iter()
                                        .map(|_| ctx.i32_type().into())
                                        .collect();

                                    let fn_ty = ret_ty.fn_type(&param_tys, false);
                                    let fn_val: FunctionValue =
                                        llvm_module.add_function(name, fn_ty, None);

                                    println!(
                                        "✅ [import] Created external stub for '{}::{}'",
                                        module, name
                                    );

                                    fn_val.set_linkage(Linkage::External);
                                }
                            } else {
                                eprintln!(
                                    "⚠️ [resolver] Missing export '{}' in module '{}'",
                                    member_name, module
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// 🧩 Runtime phase: bind all functions to their compiled addresses.
    /// This avoids segfaults from unresolved calls.
    pub fn link_imports_runtime<'ctx>(
        &self,
        engine: &ExecutionEngine<'ctx>,
        llvm_module: &Module<'ctx>,
        wms: &ModuleSystem,
    ) {
        let cache = wms.get_cache();

        for (module_name, module_data) in cache.iter() {
            for node in &module_data.ast {
                match node {
                    Node::Export { name, item } => {
                        if let Node::Expr(Expr::Funcy { name: fn_name, .. }) = &**item {
                            let symbol_name = fn_name.clone();

                            // Get compiled address from JIT
                            if let Ok(addr) = engine.get_function_address(&symbol_name) {
                                if addr != 0 {
                                    if let Some(llvm_func) = llvm_module.get_function(&symbol_name) {
                                        unsafe {
                                            engine.add_global_mapping(&llvm_func, addr as usize);
                                        }
                                        println!(
                                            "✅ [runtime-link] Bound '{}' from module '{}' at 0x{:X}",
                                            symbol_name, module_name, addr
                                        );
                                    }
                                }
                            } else {
                                eprintln!(
                                    "⚠️ [runtime-link] Missing runtime address for '{}'",
                                    symbol_name
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
