using System.Text.Json;
using WPlusPlus;

class Program
{
    static async Task Main(string[] args)
    {
        if (args.Length == 0)
        {
            Console.WriteLine("Usage: ingot <command> [--jit]");
            return;
        }

        if (args[0] == "--help" || args[0] == "help")
{
    Console.WriteLine("Ingot CLI v0.2.2");
    Console.WriteLine("Usage:");
    Console.WriteLine("  ingot init                  Create a new W++ project");
    Console.WriteLine("  ingot run [--jit]           Run the W++ project (optionally with JIT)");
    Console.WriteLine("  ingot build                 Compile without running");
    Console.WriteLine("  ingot publish               Package build output");
    Console.WriteLine("  ingot import <package>      Import a NuGet package as a W++ ingot");
    Console.WriteLine("  ingot install               Install all ingots listed in wpp.json");
    Console.WriteLine("  ingot list                  List all currently installed ingots");
    Console.WriteLine("  ingot remove <package>      Remove an ingot and update wpp.json");
    Console.WriteLine("  ingot help                  Show this help message");
    Console.WriteLine("  ingot version               Show the current CLI version");
    return;
}


        if (args[0] == "--version" || args[0] == "version")
        {
            Console.WriteLine("Ingot CLI v0.2.2");
            return;
        }

        switch (args[0])
        {
            case "init":
                InitProject();
                break;
            case "run":
                bool forceJit = args.Contains("--jit");
                await RunProject(forceJit);
                break;
            case "build":
                await BuildProject();
                break;
            case "publish":
                PublishProject();
                break;
                case "import":
        if (args.Length < 2)
            {
            Console.WriteLine("❌ Please specify a NuGet package name.");
            return;
            }
            var package = args[1];
            await NugetIngotConverter.ImportAsync(package);
            break;
            case "install":
    await NugetIngotConverter.InstallAllAsync();
    break;

case "list":
    NugetIngotConverter.ListInstalled();
    break;

case "remove":
    if (args.Length < 2)
    {
        Console.WriteLine("❌ Please specify an ingot to remove.");
        return;
    }
    NugetIngotConverter.RemoveIngot(args[1]);
    break;


            default:
                Console.WriteLine("Unknown command.");
                break;
        }
    }

    static void InitProject()
    {
        File.WriteAllText("main.wpp", "// Your W++ code here\nprint \"Hello, world!\";");
        File.WriteAllText("wpp.json", JsonSerializer.Serialize(new
        {
            name = "myproject",
            version = "0.1.0",
            main = "main.wpp",
            jit = false,
            dependencies = new Dictionary<string, string>()
        }, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine("✅ Project initialized.");
    }

    static async Task RunProject(bool forceJit = false)
    {
        if (!File.Exists("wpp.json"))
        {
            Console.WriteLine("❌ wpp.json not found. Run 'ingot init' first.");
            return;
        }

        var json = JsonDocument.Parse(File.ReadAllText("wpp.json"));
        string entry = json.RootElement.GetProperty("main").GetString();
        bool configJit = json.RootElement.TryGetProperty("jit", out var jitProp) && jitProp.GetBoolean();
        bool useJit = forceJit || configJit;

        if (!File.Exists(entry))
        {
            Console.WriteLine($"❌ Entry file '{entry}' not found.");
            return;
        }

        string code = File.ReadAllText(entry);
        var tokens = Lexer.Tokenize(code);
        var parser = new Parser(tokens);

        if (useJit)
        {
            Console.ForegroundColor = ConsoleColor.Yellow;
            Console.WriteLine("🚀 Running JIT compiled W++ code (experimental!)...");
            Console.ResetColor();

            var jit = new JitCompiler();
            var ast = parser.Parse();
            jit.Compile(ast);
        }
        else
        {
            Console.WriteLine("🌀 Running W++ with interpreter...");
            var interpreter = new Interpreter();

            while (parser.HasMore())
            {
                var node = parser.Parse();
                await interpreter.Evaluate(node);
            }

            Console.WriteLine("✅ Execution finished.");
        }
    }

    static async Task BuildProject()
    {
        if (!File.Exists("wpp.json"))
        {
            Console.WriteLine("❌ wpp.json not found. Run 'ingot init' first.");
            return;
        }

        var json = JsonDocument.Parse(File.ReadAllText("wpp.json"));
        string entry = json.RootElement.GetProperty("main").GetString();

        if (!File.Exists(entry))
        {
            Console.WriteLine($"❌ Entry file '{entry}' not found.");
            return;
        }

        string outputDir = "build";
        Directory.CreateDirectory(outputDir);

        string fullCode = File.ReadAllText(entry);
        File.WriteAllText(Path.Combine(outputDir, "bundle.wpp"), fullCode);

        Console.WriteLine("📦 Build complete → build/bundle.wpp");
    }

    static void PublishProject()
    {
        string source = Path.Combine("build", "bundle.wpp");
        string targetDir = "dist";
        string target = Path.Combine(targetDir, "bundle.wppack");

        if (!File.Exists(source))
        {
            Console.WriteLine("❌ No bundle to publish. Run 'ingot build' first.");
            return;
        }

        Directory.CreateDirectory(targetDir);
        File.Copy(source, target, overwrite: true);

        Console.WriteLine($"🚀 Published to {target}");
    }
}
