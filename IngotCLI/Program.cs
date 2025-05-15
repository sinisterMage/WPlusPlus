using System.Text.Json;
using WPlusPlus;

class Program
{
    static async Task Main(string[] args)
    {
        if (args.Length == 0)
        {
            Console.WriteLine("Usage: ingot <command>");
            return;
        }
        if (args.Length == 1 && (args[0] == "--help" || args[0] == "help"))
        {
            Console.WriteLine("Ingot CLI v0.1.0");
            Console.WriteLine("Usage:");
            Console.WriteLine("  ingot init        Create a new W++ project");
            Console.WriteLine("  ingot run         Run the W++ project");
            Console.WriteLine("  ingot build       Compile without running");
            Console.WriteLine("  ingot publish     Package build output");
            Console.WriteLine("  ingot help        Show this help message");
            return;
        }

        if (args.Length == 1 && (args[0] == "--version" || args[0] == "version"))
        {
            Console.WriteLine("Ingot CLI v0.1.0");
            return;
        }


        switch (args[0])
        {
            case "init":
                InitProject();
                break;
            case "run":
                await RunProject();
                break;
            case "build":
                await BuildProject();
                break;
            case "publish":
                PublishProject();
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
            dependencies = new Dictionary<string, string>()
        }, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine("✅ Project initialized.");
    }

    static async Task RunProject()
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

        string code = File.ReadAllText(entry);
        var tokens = Lexer.Tokenize(code);
        var parser = new Parser(tokens);
        var interpreter = new Interpreter();

        while (parser.HasMore())
        {
            var node = parser.Parse();
            await interpreter.Evaluate(node);
        }

        Console.WriteLine("✅ Execution finished.");
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

        // You can enhance this later to inline all imports if needed.
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
