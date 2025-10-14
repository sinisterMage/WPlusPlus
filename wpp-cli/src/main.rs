use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use colored::*;
use wpp_v2::{run_file, build_ir};
use wpp_v2::lexer::Lexer;
use wpp_v2::codegen::Codegen;
use inkwell::context::Context;

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
        let config_text = fs::read_to_string(&config_path).unwrap_or_default();
        let entry_re = Regex::new(r#"entrypoint\s+"([^"]+)""#).unwrap();
        let entry_path = entry_re
            .captures(&config_text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "src/main.wpp".to_string());

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

        return;
    }

    // === Single-File Mode ===
    let file_path = Path::new(path);
    if !file_path.exists() {
        eprintln!("❌ File not found: {path}");
        return;
    }

    println!("🚀 Running {path}...\n");

    match fs::read_to_string(file_path) {
        Ok(source) => run_with_codegen(&source, optimize),
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
fn run_with_codegen(source: &str, optimize: bool) {
    use wpp_v2::{lexer::Lexer, parser::Parser, codegen::Codegen, run_file};
    use inkwell::context::Context;

    // 🧠 Step 1: Create LLVM context once
    let context = Context::create();

    // 🧱 Step 2: Lex + parse
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let nodes = parser.parse_program();

    // ⚙️ Step 3: Generate IR
    let mut codegen = Codegen::new(&context, "wpp_module");
    codegen.compile_main(&nodes);

    // 🧠 Step 4: Run JIT (unified API)
    match run_file(&mut codegen, optimize) {
        Ok(_) => println!("✅ Execution finished successfully."),
        Err(e) => eprintln!("❌ Error during execution: {e}"),
    }
}
