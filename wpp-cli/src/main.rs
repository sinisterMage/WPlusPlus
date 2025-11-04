use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::{get, Client};
use tokio::time::Instant;
use wpp_v2::export_resolver::ExportResolver;
use wpp_v2::module_system::ModuleSystem;
use std::fs::{self, File, OpenOptions};
use std::io::{self, copy, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{env, thread};
use std::time::Duration;
use colored::*;
use wpp_v2::{run_file, build_ir};
use wpp_v2::lexer::Lexer;
use wpp_v2::parser::Parser as WppParser;
use wpp_v2::codegen::Codegen;
use inkwell::context::Context;
mod config;
use crate::config::WppConfig;
mod api;
use api::*;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
mod wms;
use wms::WmsResolver;

/// Helper function to detect file metadata for Rust interop libraries
/// Returns: (file_type, platform, architecture)
fn detect_file_metadata(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let ext = path.extension().and_then(|s| s.to_str());

    let file_type = match ext {
        Some("wpp") => Some("source".to_string()),
        Some("dylib") | Some("so") | Some("dll") => Some("library".to_string()),
        Some("rs") => Some("rust-source".to_string()),
        Some("toml") => Some("config".to_string()),
        _ => None,
    };

    let platform = match ext {
        Some("dylib") => Some("darwin".to_string()),
        Some("so") => Some("linux".to_string()),
        Some("dll") => Some("windows".to_string()),
        _ => None, // Source files are platform-independent
    };

    // Capture architecture for binary files
    let architecture = if matches!(ext, Some("dylib" | "so" | "dll")) {
        Some(std::env::consts::ARCH.to_string()) // "x86_64", "aarch64", etc.
    } else {
        None
    };

    (file_type, platform, architecture)
}

/// 🦥 Ingot CLI — Chaos meets LLVM
#[derive(Parser)]
#[command(name = "ingot", version, about = "W++ LLVM CLI — Run, Build, and Cause Chaos")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a W++ source file using the LLVM JIT
    Run {
    /// Optional path to a W++ file (if omitted, project mode is used)
    file: Option<String>,

    /// Enable LLVM optimization passes
    #[arg(short, long)]
    opt: bool,
    
},


    /// Build a W++ source file into LLVM IR (.ll)
    Build {
        /// Path to the W++ file
        file: String,

        /// Output path for the LLVM IR
        #[arg(short, long, default_value = "out.ll")]
        output: String,

        /// Enable LLVM optimization passes
        #[arg(short, long)]
        opt: bool,
    },

    /// The classic troll: npm install 69,000 packages
    NpmInstall,

    /// The Arch btw troll command
    Pacman,

    /// Show help and available commands
    Info,
    Init {
        /// Project name (optional; defaults to current folder)
        name: Option<String>,
    },
        /// Verify your Ingot registry token
    Login,

    /// Install a package from the Ingot registry
    Install {
    /// Package name
    name: String,

    /// Version (defaults to latest if omitted)
    #[arg(default_value = "latest")]
    version: String,
},

    /// Publish a package to the registry
    Publish,
        Fetch,

}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, opt } => {
    if let Some(f) = file {
        run_file_command(&f, opt);
    } else {
        run_file_command(".", opt); // treat "." as project root
    }
}
        Commands::Build { file, output, opt } => build_file_command(&file, &output, opt),
        Commands::NpmInstall => troll_npm_install(),
        Commands::Pacman => troll_pacman(),
        Commands::Info => print_help(),
        Commands::Init { name } => init_project(name),
                Commands::Login => {
    use std::{fs, path::PathBuf};
    use rpassword::read_password;
    use colored::*;

    // 🧠 Step 1: Try to load from env or saved file
    let mut token = std::env::var("WPP_TOKEN").ok();

    if token.is_none() {
        let mut path = dirs::home_dir().unwrap_or_default();
        path.push(".wpp_token");

        if path.exists() {
            token = fs::read_to_string(&path).ok().map(|t| t.trim().to_string());
        }
    }

    // 🧩 Step 2: If still missing, ask user to input it
    if token.is_none() {
        println!("🔐 Please enter your W++ Ingot API token (input hidden):");
        match read_password() {
            Ok(input) if !input.trim().is_empty() => {
                token = Some(input.trim().to_string());
                // Save locally for next time
                let mut path = dirs::home_dir().unwrap_or_default();
                path.push(".wpp_token");
                if let Err(e) = fs::write(&path, token.as_ref().unwrap()) {
                    eprintln!("⚠️ Failed to save token locally: {e}");
                } else {
                    println!("💾 Token saved to {}", path.display());
                }
            }
            _ => {
                eprintln!("❌ No token entered. Aborting login.");
                return;
            }
        }
    }

    // 🛰️ Step 3: Verify login
    let api = IngotAPI::new("https://ingotwpp.dev", token.clone());

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(api.verify_login())
    {
        Ok(res) => {
            if res.success {
                if let Some(user) = res.user {
                    println!(
                        "✅ Logged in as {} {}",
                        user.firstName.green(),
                        user.lastName.green()
                    );
                } else {
                    println!("✅ Login verified: {}", res.message.green());
                }
            } else {
                println!("❌ Login failed: {}", res.message.red());
            }
        }
        Err(e) => eprintln!("❌ Login request failed: {e}"),
    }
}


      Commands::Install { name, version } => {
    let token = std::env::var("WPP_TOKEN").ok();
    let api = IngotAPI::new("https://ingotwpp.dev", token);

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(api.get_package(&name, &version))
    {
        Ok(pkg) => {
            println!("📦 Installing {} v{}", pkg.name, pkg.version);

            let install_dir = Path::new(".wpp_packages").join(&pkg.name);
            fs::create_dir_all(&install_dir).unwrap();

            // Cache directory in ~/.wpp_cache
            let cache_dir = dirs::home_dir()
                .unwrap()
                .join(".wpp_cache")
                .join(&pkg.name)
                .join(&pkg.version);
            fs::create_dir_all(&cache_dir).unwrap();

            let client = Client::new();
            let start = Instant::now();
            let mut total_bytes_downloaded = 0u64;

            for f in pkg.files {
                // === Platform Compatibility Check ===
                // Skip binaries that don't match current platform
                if f.fileType.as_deref() == Some("library") {
                    if let Some(file_platform) = &f.platform {
                        let current_platform = std::env::consts::OS;
                        if file_platform != current_platform {
                            println!("⏭️  Skipping {} (platform: {}, current: {})",
                                f.filename.dimmed(), file_platform, current_platform);
                            continue;
                        }
                    }
                }

                // === Determine Installation Path ===
                // Libraries go to rust_modules/, other files to project root
                let dest_path = if f.fileType.as_deref() == Some("library") {
                    let rust_modules_dir = install_dir.join("rust_modules");
                    fs::create_dir_all(&rust_modules_dir).unwrap();
                    rust_modules_dir.join(&f.filename)
                } else {
                    install_dir.join(&f.filename)
                };

                let cache_path = cache_dir.join(&f.filename);

                // === Step 1: Check cache first
                if cache_path.exists() {
                    println!("💾 Using cached {} {}", f.filename,
                        if f.fileType.as_deref() == Some("library") { "[library]".yellow() } else { "".normal() });
                    fs::copy(&cache_path, &dest_path).unwrap();
                    continue;
                }

                // === Step 2: Download if not cached
                println!("⬇️ Downloading {}...", f.filename);

                let mut req = client.get(&f.downloadUrl);
                if let Some((k, v)) = api.auth_header() {
                    req = req.header(k, v);
                }

                match req.send() {
                    Ok(mut resp) => {
                        if !resp.status().is_success() {
                            eprintln!(
                                "❌ Failed to download {} (status: {})",
                                f.filename,
                                resp.status()
                            );
                            continue;
                        }

                       let total_size = resp.content_length().unwrap_or(0);
let pb = ProgressBar::new(total_size);
pb.set_style(
    ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
    )
    .unwrap()
    .progress_chars("#>-"),
);

// Use the dest_path and cache_path already determined above
// (dest_path already points to rust_modules/ for libraries)
fs::create_dir_all(dest_path.parent().unwrap()).unwrap();

// Create output file and wrap with progress bar
let mut dest_file = File::create(&dest_path).unwrap();
let mut writer = pb.wrap_write(&mut dest_file);

// Stream response into file + progress bar
let mut content = resp;
let bytes_written = copy(&mut content, &mut writer).unwrap_or(0);
total_bytes_downloaded += bytes_written;

// Compute checksum for integrity validation
let file_bytes = fs::read(&dest_path).unwrap();
let checksum = checksum_bytes(&file_bytes);

// Write to cache and finalize progress bar
fs::write(&cache_path, &file_bytes).unwrap();
pb.finish_and_clear();

println!("✅ Saved {}", dest_path.display());

// Update .simula lock file
update_simula_lock(&pkg.name, &pkg.version, &checksum).unwrap();


                        // Update .simula lock file
                        update_simula_lock(&pkg.name, &pkg.version, &checksum).unwrap();
                    }
                    Err(e) => eprintln!("❌ Error downloading {}: {}", f.filename, e),
                }
            }

            let elapsed = start.elapsed();
            println!(
                "\n✨ Installed {} v{} in {:.2?} ({} bytes)\n",
                pkg.name,
                pkg.version,
                elapsed,
                total_bytes_downloaded
            );
        }
        Err(e) => eprintln!("❌ Failed to install package: {e}"),
    }
}

        Commands::Publish => {
    let token = std::env::var("WPP_TOKEN").ok();
    let api = IngotAPI::new("https://ingotwpp.dev", token);

    println!("🔍 Loading wpp.config.hs...");
    let cfg = match WppConfig::load(Path::new("wpp.config.hs")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load config: {e}");
            return;
        }
    };

    let meta = PublishMetadata {
        name: cfg.project_name.clone().unwrap_or_else(|| "unnamed".into()),
        version: cfg.version.clone().unwrap_or_else(|| "0.1.0".into()),
        description: Some("Published via W++ Ingot CLI".into()),
        license: cfg.license.clone(),
        category: Some("utilities".into()),
        tags: Some(cfg.flags.clone()),
        readme: Some("# Published via W++ Ingot CLI".into()),
        is_public: Some(true),
    };

    println!("📦 Package: {}", meta.name);
    println!("🧩 Version: {}", meta.version);
    if let Some(lic) = &meta.license {
        println!("📜 License: {}", lic);
    }

    // 🔎 Collect all project files recursively (.wpp, .dylib, .so, .dll, .rs, .toml)
    println!("🗂️  Scanning project files...");
    let mut files = Vec::new();
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            if !e.file_type().is_file() {
                return false;
            }
            // Include .wpp source files and Rust interop files
            let ext = e.path().extension().and_then(|s| s.to_str());
            matches!(ext, Some("wpp" | "dylib" | "so" | "dll" | "rs" | "toml"))
        })
    {
        let path = entry.path();
        let (file_type, platform, architecture) = detect_file_metadata(path);

        files.push(FileWithMetadata {
            path: path.display().to_string(),
            file_type,
            platform,
            architecture,
        });
    }

    if files.is_empty() {
        eprintln!("❌ No project files found (.wpp, .dylib, .so, .dll, .rs, .toml).");
        return;
    }

    println!("📄 Found {} files for upload:", files.len());
    for f in &files {
        let type_info = f.file_type.as_ref().map(|t| format!(" [{}]", t)).unwrap_or_default();
        let platform_info = f.platform.as_ref().map(|p| format!(" ({}, {})", p, f.architecture.as_ref().unwrap_or(&"unknown".to_string()))).unwrap_or_default();
        println!("  - {}{}{}", f.path.bright_cyan(), type_info.yellow(), platform_info.dimmed());
    }

    println!("🚀 Uploading package...");
    show_upload_progress();

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(api.publish_package(&meta, files))
    {
        Ok(_) => println!(
            "✅ Successfully published {} v{} to the Ingot registry!",
            meta.name, meta.version
        ),
        Err(e) => eprintln!("❌ Publish failed: {e}"),
    }
}

Commands::Fetch => {
    let token = std::env::var("WPP_TOKEN").ok();
    let api = Arc::new(IngotAPI::new("https://ingotwpp.dev", token));
    let wms = WmsResolver::new(api);
    if let Err(e) = wms.resolve_all(Path::new(".")) {
        eprintln!("❌ Dependency resolution failed: {e}");
    }
}

    }
}

/// 🚀 Run a W++ file using the LLVM JIT
/// 🚀 Run a W++ file or project using the LLVM JIT
fn run_file_command(path: &str, optimize: bool) {
    use regex::Regex;
    use std::path::Path;
    use inkwell::context::Context;
    use wpp_v2::{lexer::Lexer, parser::Parser, codegen::Codegen, run_file};

    // Detect project root
    let current_dir = std::env::current_dir().unwrap();
    let config_path = current_dir.join("wpp.config.hs");

    // === Project Mode ===
    if config_path.exists() {
        println!("🔮 Detected wpp.config.hs → loading functional configuration...");

        // Try to extract entrypoint from config
        // 🔧 Load functional config through structured parser
match WppConfig::load(&config_path) {
    Ok(cfg) => {

        if let Some(name) = &cfg.project_name {
            println!("📦 Package: {name}");
        }
        if let Some(ver) = &cfg.version {
            println!("🧩 Version: {ver}");
        }
        if let Some(lic) = &cfg.license {
            println!("📜 License: {lic}");
        }
        if let Some(auth) = &cfg.author {
            println!("👤 Author: {auth}");
        }

        for msg in &cfg.messages {
            println!("💬 {msg}");
        }

        for flag in &cfg.flags {
            println!("🧠 Applied flag from config: {flag}");
        }

        // 🧩 Resolve dependencies if any are declared
        let deps: Vec<String> = cfg.flags.iter()
            .filter(|f| f.starts_with("--dep="))
            .map(|f| f.trim_start_matches("--dep=").to_string())
            .collect();

        if !deps.is_empty() {
            println!("📦 Resolving {} dependencies...", deps.len());
            let token = std::env::var("WPP_TOKEN").ok();
            let api = Arc::new(IngotAPI::new("https://ingotwpp.dev", token));
            let wms_resolver = WmsResolver::new(api);
            if let Err(e) = wms_resolver.resolve_all(&current_dir) {
                eprintln!("⚠️  Warning: Dependency resolution failed: {e}");
                eprintln!("    Continuing anyway - dependencies may not be available.");
            }
        }

        let entry_path = cfg.entrypoint.unwrap_or_else(|| "src/main.wpp".to_string());
        let entry_full = current_dir.join(&entry_path);

        if !entry_full.exists() {
            eprintln!("❌ Entrypoint not found: {}", entry_full.display());
            return;
        }

        println!("📦 Project root: {}", current_dir.display());
        println!("▶️  Entrypoint: {}\n", entry_path);

        match fs::read_to_string(&entry_full) {
            Ok(source) => run_with_codegen(&source, optimize),
            Err(e) => eprintln!("❌ Could not read entrypoint: {e}"),
        }
    }
    Err(e) => eprintln!("❌ Failed to load wpp.config.hs: {e}"),
}



        return;
    }

    // === Single-File Mode ===
    let file_path = Path::new(path);
    if !file_path.exists() {
        eprintln!("❌ File not found: {path}");
        return;
    }

    println!("🚀 Running {path} (single-file) ...\n");

    match fs::read_to_string(file_path) {
        Ok(source) => {
            // Use the directory of the file as the base for codegen (for any relative needs)
            let base_dir_buf = file_path.parent().map(|p| p.to_path_buf());
            let base_dir = base_dir_buf
                .as_ref()
                .and_then(|p| p.to_str())
                .unwrap_or(".");
            run_single_source(&source, base_dir, optimize);
        }
        Err(e) => eprintln!("❌ Could not read file: {e}"),
    }
}

/// 🏗️ Build a W++ file into LLVM IR (.ll)
fn build_file_command(path: &str, output: &str, optimize: bool) {
    println!("🏗️  Building {path} → {output}...");

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Could not read file: {}", e);
            return;
        }
    };

    match build_ir(&source, optimize) {
        Ok(ir) => {
            if let Err(e) = fs::write(output, ir) {
                eprintln!("❌ Failed to write LLVM IR: {}", e);
                return;
            }
            println!("✅ LLVM IR written to {output}");
        }
        Err(e) => eprintln!("❌ Build failed: {}", e),
    }
}

/// 💀 npm install troll
fn troll_npm_install() {
    println!("ok, installing 69,000 packages into node_modules...");

    let node_modules = Path::new("node_modules");
    fs::create_dir_all(node_modules).unwrap();

    let lock_file = node_modules.join("package-lock.wpp");
    fs::write(&lock_file, "warning, sanity not found, please call 1-800-WLOTH").unwrap();

    for i in 0..5 {
        println!("Installing package {}...", (i + 1) * 1337);
        thread::sleep(Duration::from_millis(300));
    }

    println!("🧠 sanity check failed: 'wloth.core' missing native bindings");
    println!("Done. Don't forget to run 'ingot audit fix --chaos'.");
}

/// 🐧 pacman troll (Arch btw)
fn troll_pacman() {
    println!("{}", ":: Synchronizing package databases...".green());
    thread::sleep(Duration::from_millis(800));

    println!("{}", ":: Starting full system wipe...".green());
    thread::sleep(Duration::from_millis(1000));

    println!("{}", "💣 ok, deleting your OS and installing Arch btw...".red());
    thread::sleep(Duration::from_millis(1200));

    fs::create_dir_all("node_modules").unwrap();
    let iso_path = Path::new("node_modules/archbtw.iso");
    println!("📥 Downloading Arch ISO (700MB of pain)...");

    let mut progress = 0.0;
    while progress < 1.0 {
        progress += 0.05;
        draw_progress_bar(progress, 40);
        thread::sleep(Duration::from_millis(100));
    }

    println!("\n✅ Arch ISO has been installed (maliciously) at: {}", iso_path.display());
    println!("✨ Welcome to the rice fields, baby.");
}

/// 🆘 Print help menu (with optimization flag info)
fn print_help() {
    // 🦥 Custom ASCII banner for "INGOT"
    println!("{}", r#"
██╗███╗   ██╗ ██████╗  ██████╗ ████████╗
██║████╗  ██║██╔═══██╗██╔═══██╗╚══██╔══╝
██║██╔██╗ ██║██║   ██║██║   ██║   ██║   
██║██║╚██╗██║██║   ██║██║   ██║   ██║   
██║██║ ╚████║╚██████╔╝╚██████╔╝   ██║   
╚═╝╚═╝  ╚═══╝ ╚═════╝  ╚═════╝    ╚═╝   
"#.bright_yellow());

    println!("{}", "W++ LLVM CLI — Chaos meets Optimization\n".bold().bright_yellow());
    println!("{}", "-------------------------------------------".bright_black());

    // 🧠 Command list
    println!("Commands:");
    println!(
        "  {} {}\n      {}",
        "run <file>".cyan(),
        "→ Run a W++ file using the LLVM JIT".bright_black(),
        "--opt / -o".bright_green().to_string() + " → Enable LLVM optimization passes"
    );

    println!(
        "  {} {}\n      {}",
        "build <file> [-o <out.ll>]".cyan(),
        "→ Compile a W++ file to LLVM IR".bright_black(),
        "--opt / -o".bright_green().to_string() + " → Enable LLVM optimization passes"
    );

    println!(
        "  {} {}",
        "npm-install".cyan(),
        "→ Install 69,000 useless packages (troll command)".bright_black()
    );

    println!(
        "  {} {}",
        "pacman".cyan(),
        "→ Delete your OS and install Arch btw (troll command)".bright_black()
    );

    println!(
        "  {} {}",
        "info".cyan(),
        "→ Show this help and usage information".bright_black()
    );

    println!("{}", "-------------------------------------------".bright_black());

    // 💡 Usage examples
    println!("Examples:");
    println!("  {}", "ingot run examples/hello.wpp".bright_green());
    println!("  {}", "ingot build examples/hello.wpp -o out.ll".bright_green());
    println!("  {}", "ingot run --opt examples/optimized.wpp".bright_green());
    println!("  {}", "ingot build --opt examples/hello.wpp -o optimized.ll".bright_green());

    println!("{}", "-------------------------------------------".bright_black());
    println!("{}", "Sloth-powered. Chaos-approved. Optimized (maybe).".italic().bright_black());
}



/// 🔧 Helper: draw progress bar
fn draw_progress_bar(progress: f32, width: usize) {
    let filled = (progress * width as f32) as usize;
    let empty = width - filled;
    print!(
        "\r[{}{}] {:>3}%",
        "=".repeat(filled),
        " ".repeat(empty),
        (progress * 100.0) as i32
    );
    io::stdout().flush().unwrap();
}
/// 🧱 Initialize a new W++ project with functional chaos config
fn init_project(name: Option<String>) {
    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "wpp_project".to_string())
    });

    let root = Path::new(&project_name);
    if root.exists() {
        println!("⚠️  Directory '{}' already exists.", project_name);
    } else {
        fs::create_dir_all(root.join("src")).unwrap();
    }

    // src/main.wpp
    let main_code = r#"print("Hello from W++!")
print("✨ Functional configuration detected!")
"#;
    fs::write(root.join("src/main.wpp"), main_code).unwrap();

    // wpp.config.hs
    let config_text = format!(
r#"-- wpp.config.hs
-- W++ Functional Configuration — Because JSON is for mortals

main :: IO ()
main = do
  entrypoint "src/main.wpp"
  package   "{name}"
  version   "1.0.0"
  license   "MIT"
  author    "Ofek Bickel"
  println  "✨ Config loaded successfully. Chaos imminent."
"#,
        name = project_name
    );
    fs::write(root.join("wpp.config.hs"), config_text).unwrap();

    // lockfile
    fs::write(root.join("ingot.lock"), "# lockfile of chaos\n").unwrap();

    // hidden build folder
    fs::create_dir_all(root.join(".wpp/cache")).unwrap();

    println!("\n✅ Initialized new W++ project: {}\n", project_name.bright_green());
    println!("📄 Created files:");
    println!("  - {}", "src/main.wpp".cyan());
    println!("  - {}", "wpp.config.hs".cyan());
    println!("  - {}", "ingot.lock".cyan());
    println!("  - {}", ".wpp/cache/".cyan());
    println!("\nRun it with:");
    println!("   {}", format!("cd {} && ingot run src/main.wpp", project_name).bright_yellow());
}
fn run_with_codegen(_source: &str, optimize: bool) {
    println!("🎯 [CLI] Entered run_with_codegen function");
    // 🧠 Step 1: Create LLVM context once
    let context = Context::create();
    let project_root = std::env::current_dir().unwrap();
    let src_dir = project_root.join("src");

    // 🧱 Step 2: Initialize module system
    let mut wms = ModuleSystem::new(&project_root);
    wms.clear_cache();

    // 🧩 Step 3: Load and compile all modules (WMS compiles 'main' internally)
    println!("🎯 [CLI] About to load 'main' module via WMS");
    wms.load_module("main").expect("Failed to load main module");
    println!("🎯 [CLI] Finished loading 'main' module");

    // 🧩 Step 4: Collect exports across cached modules
    let mut resolver = ExportResolver::new();
    println!("🔍 [CLI] About to collect exports from WMS with {} cached modules", wms.get_cache().len());
    #[cfg(debug_assertions)]
    println!("📦 [debug] WMS cache keys: {:?}", wms.get_cache().keys());
    resolver.collect_exports(&wms);
    println!("🔍 [CLI] Finished collecting exports, resolver has {} symbols", resolver.global_table.len());

    // ⚙️ Step 5: Load main module from source and parse
    let main_source_path = project_root.join("src/main.wpp");
    let main_source = std::fs::read_to_string(&main_source_path)
        .expect("Failed to read main source");

    let mut lexer = Lexer::new(&main_source);
    let tokens = lexer.tokenize();
    let mut parser = WppParser::new(tokens);
    let main_ast = parser.parse_program();

    // Initialize the top-level codegen shell
    let mut codegen = Codegen::new(&context, "main", src_dir.to_str().unwrap());
    codegen.wms = Some(Arc::new(Mutex::new(wms)));
    codegen.resolver = Some(Arc::new(Mutex::new(resolver)));

    // 🔗 Step 5.5: Link dependency modules BEFORE compiling main
    println!("🔗 Linking dependency modules into main context...");
    codegen.link_dependency_modules();

    // 🧩 Step 6: Compile main module with exports available
    println!("🧩 Compiling main module with resolved exports...");
    codegen.compile_main(&main_ast);

    // 🧠 Step 7: Run via JIT
    match run_file(&mut codegen, optimize) {
        Ok(_) => println!("✅ Execution finished successfully."),
        Err(e) => eprintln!("❌ Error during execution: {e}"),
    }
}

/// 🚀 Run a single W++ source buffer directly (no project/WMS)
fn run_single_source(source: &str, base_dir: &str, optimize: bool) {
    println!("🎯 [CLI] Single-file mode: compiling buffer (base_dir = {})", base_dir);

    // Create LLVM context
    let context = inkwell::context::Context::create();

    // Lex/parse
    let mut lexer = wpp_v2::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = wpp_v2::parser::Parser::new(tokens);
    let ast = parser.parse_program();

    // Codegen + JIT
    let mut codegen = wpp_v2::codegen::Codegen::new(&context, "single", base_dir);
    codegen.compile_main(&ast);

    match wpp_v2::run_file(&mut codegen, optimize) {
        Ok(_) => println!("✅ Execution finished successfully."),
        Err(e) => eprintln!("❌ Error during execution: {e}"),
    }
}




fn show_upload_progress() {
    let symbols = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
    for _ in 0..12 {
        for s in symbols {
            print!("\r{} Uploading to Ingot registry...", s.bright_magenta());
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(80));
        }
    }
    println!("\r✅ Upload complete!                                      ");
}
fn checksum_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

pub fn update_simula_lock(pkg_name: &str, version: &str, checksum: &str) -> std::io::Result<()> {
    let path = Path::new(".simula");

    // Create .simula if missing
    if !path.exists() {
        let mut file = File::create(path)?;
        writeln!(file, "BEGIN SYSTEM SIMULATION;")?;
        writeln!(file, "   CLASS PACKAGE;")?;
        writeln!(file, "      STRING name;")?;
        writeln!(file, "      STRING version;")?;
        writeln!(file, "      STRING checksum;")?;
        writeln!(file, "   END;")?;
        writeln!(file)?;
        writeln!(file, "   BEGIN MAIN;")?;
        writeln!(file, "   END;")?;
        writeln!(file)?;
        writeln!(file, "END SYSTEM;")?;
    }

    // Read existing file
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;

    let package_block_start = format!("PACKAGE {}", pkg_name);

    if content.contains(&package_block_start) {
        println!("🔄 Updating existing package '{}' in .simula...", pkg_name);

        // Update version and checksum using simple replacements
        let updated = content
            .lines()
            .map(|line| {
                if line.trim_start().starts_with(&format!("{}.version", pkg_name)) {
                    format!("         {}.version := \"{}\";", pkg_name, version)
                } else if line.trim_start().starts_with(&format!("{}.checksum", pkg_name)) {
                    format!("         {}.checksum := \"{}\";", pkg_name, checksum)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(path, updated)?;
    } else {
        // Append new entry
        println!("➕ Adding new package '{}' to .simula...", pkg_name);
        let mut file = OpenOptions::new().write(true).append(true).open(path)?;

        writeln!(file)?;
        writeln!(file, "   BEGIN MAIN;")?;
        writeln!(file, "      PACKAGE {};", pkg_name)?;
        writeln!(file, "         {}.name := \"{}\";", pkg_name, pkg_name)?;
        writeln!(file, "         {}.version := \"{}\";", pkg_name, version)?;
        writeln!(file, "         {}.checksum := \"{}\";", pkg_name, checksum)?;
        writeln!(file, "      END;")?;
        writeln!(file, "   END;")?;
    }

    Ok(())
}
