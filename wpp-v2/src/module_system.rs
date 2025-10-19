//! W++ Module System (WMS)
//! Handles multi-threaded module resolution, parsing, and caching of imported modules.

use crate::ast::Node;
use crate::parser::parse;
use inkwell::module::{Linkage, Module};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Represents a parsed module stored in cache.
#[derive(Clone)]
pub struct ModuleData {
    pub name: String,
    pub path: PathBuf,
    pub ast: Vec<Node>,
    pub llvm_ir: Option<String>,
}

/// Thread-safe, caching, multi-threaded module system for W++.
pub struct ModuleSystem {
    base_dir: PathBuf,
    cache: Arc<Mutex<HashMap<String, ModuleData>>>,
}

impl ModuleSystem {
    // =========================================================
    // 🧱 Initialization
    // =========================================================

    /// Create a new module system, centered at the given base directory.
    pub fn new<P: Into<PathBuf>>(base_dir: P) -> Self {
        let base = base_dir.into();
        Self {
            base_dir: base,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // =========================================================
    // 📦 Public API
    // =========================================================

    /// Load a single module (from cache if possible).
   pub fn load_module(&self, name: &str) -> Result<Vec<Node>, String> {
    use crate::Codegen;
    use inkwell::context::Context;

    // === 1️⃣ Prevent infinite recursion ===
    {
        let cache = self.cache.lock().unwrap();
        if let Some(existing) = cache.get(name) {
            if existing.ast.is_empty() {
                eprintln!("⚠️ [wms] Circular import detected for '{}'", name);
                return Ok(vec![]);
            }
            return Ok(existing.ast.clone());
        }
    }

    // === 2️⃣ Resolve & read source ===
    let resolved_path = self.resolve_module_path(name)?;
    let source = std::fs::read_to_string(&resolved_path)
        .map_err(|e| format!("Failed to read module '{}': {}", resolved_path.display(), e))?;

    println!("📦 [wms] Loading module '{}'", name);

    // === 3️⃣ Parse into AST ===
    let ast = parse(&source)?;

    // Insert placeholder to prevent circular recursion
    self.cache.lock().unwrap().insert(
        name.to_string(),
        ModuleData {
            name: name.to_string(),
            path: resolved_path.clone(),
            ast: vec![],
            llvm_ir: None,
        },
    );

    // === 4️⃣ Recursively load dependencies ===
    for node in &ast {
        match node {
            Node::ImportList { module, .. } | Node::ImportAll { module } => {
                let mut cache = self.cache.lock().unwrap();
                if !cache.contains_key(module) {
                    drop(cache); // unlock before recursive call
                    #[cfg(debug_assertions)]
                    println!("🔗 [wms] Auto-loading dependency '{}'", module);
                    if let Err(e) = self.load_module(module) {
                        eprintln!("⚠️ [wms] Failed to load dependency '{}': {}", module, e);
                    }
                }
            }
            _ => {}
        }
    }

    // === 5️⃣ Compile LLVM IR ===
    let context = Context::create();
let mut codegen = Codegen::new(&context, name, "./src");

// 🧱 Compile the module first
codegen.compile_main(&ast);

if name != "main" {
    for gname in ["_wpp_exc_flag", "_wpp_exc_i32", "_wpp_exc_str"] {
        if let Some(global) = codegen.module.get_global(gname) {
            // Force external linkage and remove its initializer
            global.set_linkage(Linkage::External);

            // LLVM doesn't let you "unset" the initializer directly; you clear it by replacing it with none via raw API
            unsafe {
                use inkwell::llvm_sys::core::LLVMSetInitializer;
                use inkwell::values::AsValueRef;
                LLVMSetInitializer(global.as_value_ref(), std::ptr::null_mut());
            }
            #[cfg(debug_assertions)]
            println!("🔧 [wms] Neutralized duplicate global '{}' in module '{}'", gname, name);
        }
    }
}


let ir_str = Some(codegen.module.print_to_string().to_string());

    // === 6️⃣ Update cache ===
    let data = ModuleData {
        name: name.to_string(),
        path: resolved_path.clone(),
        ast: ast.clone(),
        llvm_ir: ir_str,
    };
    self.cache.lock().unwrap().insert(name.to_string(), data);

    println!("✅ Cached compiled module '{}'", name);
    Ok(ast)
}



    /// Load multiple modules in parallel using Rayon.
    pub fn load_modules_parallel(&self, modules: &[String]) -> Result<Vec<Vec<Node>>, String> {
        modules
            .par_iter()
            .map(|m| self.load_module(m))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Returns all cached modules (for debugging or hot-reload).
    pub fn list_cached_modules(&self) -> Vec<String> {
        self.cache
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    }

    // =========================================================
    // 🧭 Path Resolution
    // =========================================================

    /// Resolves a module name to a `.wpp` file path.
    ///
    /// Supports:
    /// - `"std/http"` → `<base_dir>/std/http.wpp`
    /// - `"./utils.wpp"` → relative path
    /// - `"foo"` → `<base_dir>/foo.wpp`
    fn resolve_module_path(&self, name: &str) -> Result<PathBuf, String> {
    use std::env;

    // Allow overrides via environment variables for flexibility
    let src_dir = env::var("WPP_SRC_PATH").unwrap_or_else(|_| "src".into());
    let pkg_dir = env::var("WPP_PACKAGES_PATH").unwrap_or_else(|_| {
    // prefer hidden .wpp_packages if it exists
    let hidden = ".wpp_packages";
    let normal = "wpp_packages";
    if Path::new(hidden).exists() {
        hidden.into()
    } else {
        normal.into()
    }
});


    // Normalize name like "math/utils"
    let normalized = name.replace('.', "/");

    // --- 1️⃣ Try explicit relative path first
    let mut path = if normalized.starts_with("./") || normalized.starts_with("../") {
        self.base_dir.join(&normalized)
    } else {
        self.base_dir.join(&src_dir).join(&normalized)
    };

    // --- 2️⃣ If directory → assume mod.wpp
    if path.is_dir() {
        path = path.join("mod.wpp");
    } else if !path.to_string_lossy().ends_with(".wpp") {
        path.set_extension("wpp");
    }

    if path.exists() {
        return Ok(path);
    }

    // --- 3️⃣ Try Ingot package: wpp_packages/<name>/main.wpp
    let pkg_main = self.base_dir
        .join(&pkg_dir)
        .join(&normalized)
        .join("main.wpp");
    if pkg_main.exists() {
        return Ok(pkg_main);
    }

    // --- 4️⃣ Try flat package: wpp_packages/<name>.wpp
    let pkg_flat = self.base_dir
        .join(&pkg_dir)
        .join(format!("{}.wpp", normalized));
    if pkg_flat.exists() {
        return Ok(pkg_flat);
    }

    // --- 5️⃣ Try stdlib fallback
    let std_path = self
        .base_dir
        .join("std")
        .join(&normalized)
        .with_extension("wpp");
    if std_path.exists() {
        return Ok(std_path);
    }

    Err(format!(
        "❌ [wms] Module '{}' not found in:\n  - {}\n  - {}\n  - {}",
        name,
        self.base_dir.join(&src_dir).display(),
        self.base_dir.join(&pkg_dir).display(),
        self.base_dir.join("std").display()
    ))
}


    // =========================================================
    // 🧩 Import Graph Utilities (optional for later)
    // =========================================================

    /// Detect circular imports using DFS traversal.
    pub fn detect_cycles(&self) -> Result<(), String> {
        let cache = self.cache.lock().unwrap();
        let mut visited = HashMap::new();

        fn dfs(
            name: &str,
            cache: &HashMap<String, ModuleData>,
            visited: &mut HashMap<String, bool>,
            stack: &mut Vec<String>,
        ) -> Result<(), String> {
            visited.insert(name.to_string(), true);
            stack.push(name.to_string());

            if let Some(module) = cache.get(name) {
                for node in &module.ast {
                    if let Node::ImportAll { module } | Node::ImportList { module, .. } = node {
                        if let Some(true) = visited.get(module) {
                            if stack.contains(module) {
                                return Err(format!(
                                    "Circular import detected: {} -> {}",
                                    stack.join(" → "),
                                    module
                                ));
                            }
                        } else {
                            dfs(module, cache, visited, stack)?;
                        }
                    }
                }
            }

            stack.pop();
            visited.insert(name.to_string(), false);
            Ok(())
        }

        for key in cache.keys() {
            let mut stack = Vec::new();
            dfs(key, &cache, &mut visited, &mut stack)?;
        }

        Ok(())
    }

    /// Clears all cached modules.
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }
}
impl ModuleSystem {
    /// Expose an immutable reference to the internal cache (for export resolver).
    pub fn get_cache(&self) -> std::sync::MutexGuard<'_, HashMap<String, ModuleData>> {
        self.cache.lock().unwrap()
    }
}
