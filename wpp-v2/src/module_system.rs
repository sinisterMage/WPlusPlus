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
use libloading::Library;

// platform extension
#[cfg(target_os = "macos")]
const LIB_EXT: &str = "dylib";
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
const LIB_EXT: &str = "so";

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
    use rayon::prelude::*;

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

    // === 2️⃣ Handle Rust native modules ===
if name.starts_with("rust:") {
    let rust_mod = name.trim_start_matches("rust:").to_string();
    println!("🦀 [wms] Loading Rust module '{}'", rust_mod);
    self.load_rust_module(&rust_mod)?;
    return Ok(vec![]);
}

// === 3️⃣ Resolve & read source ===
let resolved_path = self.resolve_module_path(name)?;
let source = std::fs::read_to_string(&resolved_path)
    .map_err(|e| format!("Failed to read module '{}': {}", resolved_path.display(), e))?;


    println!("📦 [wms] Loading module '{}'", name);

    // === 3️⃣ Parse into AST ===
    let ast = parse(&source)?;

    // Insert placeholder early to block circular recursion
    self.cache.lock().unwrap().insert(
        name.to_string(),
        ModuleData {
            name: name.to_string(),
            path: resolved_path.clone(),
            ast: vec![],
            llvm_ir: None,
        },
    );

    // === 4️⃣ Gather dependency module names ===
    let deps: Vec<String> = ast
        .iter()
        .filter_map(|node| match node {
            Node::ImportList { module, .. } | Node::ImportAll { module } => Some(module.clone()),
            _ => None,
        })
        .collect();

    // === 5️⃣ Compile dependencies concurrently ===
    if !deps.is_empty() {
        println!("🧩 [wms] Compiling {} dependencies in parallel…", deps.len());
        deps.par_iter().for_each(|dep_name| {
            // Skip if already cached
            if self.cache.lock().unwrap().contains_key(dep_name) {
                return;
            }

            // Each dependency compiles in its own LLVM context
            match self.load_module(dep_name) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    println!("✅ [wms] Finished dependency '{}'", dep_name);
                }
                Err(e) => eprintln!("⚠️ [wms] Failed to load dependency '{}': {}", dep_name, e),
            }
        });
    }

    // === 6️⃣ Compile this module to LLVM IR (skip main module body compilation) ===
    let ir_str = if name == "main" {
        // For main module, we only parse and cache the AST
        // The actual compilation will happen in the CLI with the real codegen
        println!("⚙️ [wms] Skipping main module compilation (will be done by CLI)");
        None
    } else {
        // For dependencies, compile to IR as usual
        let context = Context::create();
        let mut codegen = Codegen::new(&context, name, "./src");
        codegen.compile_main(&ast);

        // Neutralize duplicate globals for submodules
        for gname in ["_wpp_exc_flag", "_wpp_exc_i32", "_wpp_exc_str"] {
            if let Some(global) = codegen.module.get_global(gname) {
                // Force external linkage and remove its initializer
                global.set_linkage(inkwell::module::Linkage::External);
                unsafe {
                    use inkwell::llvm_sys::core::LLVMSetInitializer;
                    use inkwell::values::AsValueRef;
                    LLVMSetInitializer(global.as_value_ref(), std::ptr::null_mut());
                }
                #[cfg(debug_assertions)]
                println!(
                    "🔧 [wms] Neutralized duplicate global '{}' in module '{}'",
                    gname, name
                );
            }
        }

        Some(codegen.module.print_to_string().to_string())
    };

    // === 7️⃣ Cache compiled result ===
    let data = ModuleData {
        name: name.to_string(),
        path: resolved_path.clone(),
        ast: ast.clone(),
        llvm_ir: ir_str,
    };

    {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(name.to_string(), data);
    }

    println!("✅ Cached compiled module '{}'", name);
    Ok(ast)
}



    /// Load multiple modules in parallel using Rayon.
    /// ⚡ Load multiple modules concurrently with full IR compilation.
/// Each module gets its own LLVM context and Codegen.
pub fn load_modules_parallel(&self, modules: &[String]) -> Result<(), String> {
    use rayon::prelude::*;

    let resolved: Vec<(String, PathBuf)> = modules
        .iter()
        .map(|m| {
            self.resolve_module_path(m)
                .map(|p| (m.clone(), p))
        })
        .collect::<Result<_, _>>()?;

    resolved.par_iter().for_each(|(name, path)| {
        match self.compile_single_module(name, path) {
            Ok(data) => {
                let mut cache = self.cache.lock().unwrap();
                cache.insert(name.clone(), data);
                println!("✅ [wms-parallel] Cached module '{}'", name);
            }
            Err(e) => {
                eprintln!("❌ [wms-parallel] Failed to compile '{}': {}", name, e);
            }
        }
    });

    Ok(())
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
    fn compile_single_module(&self, name: &str, path: &Path) -> Result<ModuleData, String> {
    use crate::Codegen;
    use inkwell::context::Context;
    use inkwell::module::Linkage;

    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read module '{}': {}", path.display(), e))?;

    let ast = crate::parser::parse(&source)?;
    let context = Context::create();
    let mut codegen = Codegen::new(&context, name, "./src");
    codegen.compile_main(&ast);

    if name != "main" {
        for g in ["_wpp_exc_flag", "_wpp_exc_i32", "_wpp_exc_str"] {
            if let Some(global) = codegen.module.get_global(g) {
                global.set_linkage(Linkage::External);
                unsafe {
                    use inkwell::llvm_sys::core::LLVMSetInitializer;
                    use inkwell::values::AsValueRef;
                    LLVMSetInitializer(global.as_value_ref(), std::ptr::null_mut());
                }
            }
        }
    }

    Ok(ModuleData {
        name: name.to_string(),
        path: path.to_path_buf(),
        ast,
        llvm_ir: Some(codegen.module.print_to_string().to_string()),
    })
}
/// 🦀 Load a native Rust module (.so/.dylib) for W++ interop.
/// The library is stored in cache so it's kept loaded for the process lifetime.
pub fn load_rust_module(&self, name: &str) -> Result<(), String> {
    use std::path::PathBuf;

    let mut lib_path = self.base_dir.join("rust_modules");
    lib_path.push(format!("lib{0}.{1}", name, LIB_EXT));

    if !lib_path.exists() {
        return Err(format!("Rust module not found: {}", lib_path.display()));
    }

    unsafe {
        let lib = Library::new(&lib_path)
            .map_err(|e| format!("Failed to load Rust library '{}': {}", lib_path.display(), e))?;

        // Keep the library handle alive by inserting into cache
        let mut cache = self.cache.lock().unwrap();
        cache.insert(
            format!("rust:{}", name),
            ModuleData {
                name: format!("rust:{}", name),
                path: lib_path.clone(),
                ast: vec![],  // native module has no AST
                llvm_ir: None,
            },
        );

        // Leak library to keep it alive globally (intentional for runtime safety)
        std::mem::forget(lib);
    }

    println!("✅ [wms] Loaded Rust module '{}'", name);
    Ok(())
}

}
impl ModuleSystem {
    /// Expose an immutable reference to the internal cache (for export resolver).
    pub fn get_cache(&self) -> std::sync::MutexGuard<'_, HashMap<String, ModuleData>> {
        self.cache.lock().unwrap()
    }
}
