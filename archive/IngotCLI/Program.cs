using System.Text.Json;
using WPlusPlus;
using WPlusPlus.Shared; // Or whatever namespace `IRuntimeLinker` is under
using IngotCLI;
using System.Reflection;
using System.Net.Http;
using System.IO;
using System.Threading;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Text;


class Program
{
    static async Task Main(string[] args)
    {
        Console.OutputEncoding = Encoding.UTF8;
        Console.InputEncoding = Encoding.UTF8;
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
        if (args.Length >= 2 && args[0] == "npm" && args[1] == "install")
        {
            RunTrollNpmInstall();
            return;
        }
        if (args.Length >= 1 && args[0] == "pacman")
{
    await RunPacmanTroll();
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
                case "run-file": // <-- NEW
        if (args.Length < 2)
        {
            Console.WriteLine("❌ Please provide a .wpp file to run.");
            return;
        }
        await RunSingleFile(args[1], args.Contains("--jit"));
        break;
            case "build":
                await BuildProject();
                break;
            case "publish":
                await PublishProject(args);
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
        Assembly.Load("Newtonsoft.Json");

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
            var runtimeLinker = new RuntimeLinker(); // assuming your class is named this
            RuntimeLinker.RegisterAssembly(typeof(string).Assembly);
            var interpreter = new Interpreter(runtimeLinker);

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

    static async Task PublishProject(string[] args)
{
    // Detect platform from argument or default to current OS
    string platformArg = args.Length >= 2 ? args[1].ToLower() : null;
    string runtimeId;

    if (string.IsNullOrEmpty(platformArg))
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            runtimeId = "win-x64";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            runtimeId = "linux-x64";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            runtimeId = "osx-x64";
        else
        {
            Console.WriteLine("❌ Unsupported OS.");
            return;
        }
    }
    else
    {
        switch (platformArg)
        {
            case "win":
                runtimeId = "win-x64";
                break;
            case "linux":
                runtimeId = "linux-x64";
                break;
            case "mac":
                runtimeId = "osx-x64";
                break;
            default:
                Console.WriteLine("❌ Unknown platform. Use win, linux, or mac.");
                return;
        }
    }

    // Read project name from wpp.json
    if (!File.Exists("wpp.json"))
    {
        Console.WriteLine("❌ wpp.json not found in the current directory.");
        return;
    }
    string projectName;
    try
    {
        var json = JsonDocument.Parse(File.ReadAllText("wpp.json"));
        projectName = json.RootElement.GetProperty("name").GetString();
        if (string.IsNullOrWhiteSpace(projectName))
        {
            Console.WriteLine("❌ Invalid or missing 'name' property in wpp.json.");
            return;
        }
    }
    catch (Exception ex)
    {
        Console.WriteLine($"❌ Failed to parse wpp.json: {ex.Message}");
        return;
    }

    // Look for WppRunner in the parent directory of IngotCLI
    string baseDir = Path.GetFullPath(Path.Combine(Directory.GetCurrentDirectory(), ".."));
    string wppRunnerPath = Path.Combine(baseDir, "WppRunner");

    if (!Directory.Exists(wppRunnerPath))
    {
        Console.WriteLine($"❌ WppRunner project not found in {wppRunnerPath}");
        Console.WriteLine("   Create it with:");
        Console.WriteLine("   dotnet new console -n WppRunner");
        return;
    }

    // Build project to get bundle.wpp
    await BuildProject();

    string bundlePath = Path.Combine(Directory.GetCurrentDirectory(), "build", "bundle.wpp");
    if (!File.Exists(bundlePath))
    {
        Console.WriteLine("❌ bundle.wpp not found. Build failed?");
        return;
    }

    // Copy bundle.wpp into WppRunner as EmbeddedScript.wpp
    string embeddedPath = Path.Combine(wppRunnerPath, "EmbeddedScript.wpp");
    File.Copy(bundlePath, embeddedPath, true);

    // Ensure EmbeddedScript.wpp is embedded in WppRunner.csproj
    string csprojPath = Path.Combine(wppRunnerPath, "WppRunner.csproj");
    var csprojContent = File.ReadAllText(csprojPath);
    if (!csprojContent.Contains("<EmbeddedResource Include=\"EmbeddedScript.wpp\""))
    {
        csprojContent = csprojContent.Replace("</Project>",
            "  <ItemGroup>\n    <EmbeddedResource Include=\"EmbeddedScript.wpp\" />\n  </ItemGroup>\n</Project>");
        File.WriteAllText(csprojPath, csprojContent);
    }

    // Output directory (inside IngotCLI/dist)
    string outputDir = Path.Combine(Directory.GetCurrentDirectory(), "dist", runtimeId);
    Directory.CreateDirectory(outputDir);

    // Build & publish command
    string publishCmd = $"dotnet publish \"{Path.Combine(wppRunnerPath, "WppRunner.csproj")}\" -r {runtimeId} --self-contained true -p:PublishSingleFile=true -o \"{outputDir}\"";

    Console.WriteLine($"🚀 Publishing for {runtimeId}...");
    Console.WriteLine($"📦 Output → {outputDir}");

    var process = new Process
    {
        StartInfo = new ProcessStartInfo
        {
            FileName = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "cmd.exe" : "bash",
            Arguments = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? $"/C {publishCmd}" : $"-c \"{publishCmd}\"",
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        }
    };

    process.OutputDataReceived += (s, e) => { if (!string.IsNullOrEmpty(e.Data)) Console.WriteLine(e.Data); };
    process.ErrorDataReceived += (s, e) => { if (!string.IsNullOrEmpty(e.Data)) Console.WriteLine("❌ " + e.Data); };

    process.Start();
    process.BeginOutputReadLine();
    process.BeginErrorReadLine();
    process.WaitForExit();

    if (process.ExitCode == 0)
    {
        Console.WriteLine("✅ Publish complete!");

        // Rename published executable to projectName
        try
        {
            string ext = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? ".exe" : "";
            string oldPath = Path.Combine(outputDir, "WppRunner" + ext);
            string newPath = Path.Combine(outputDir, projectName + ext);

            if (File.Exists(oldPath))
            {
                if (File.Exists(newPath))
                    File.Delete(newPath);

                File.Move(oldPath, newPath);
                Console.WriteLine($"📦 Renamed output to {projectName}{ext}");
            }
            else
            {
                Console.WriteLine("⚠ Could not find the compiled runner to rename.");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"⚠ Failed to rename output: {ex.Message}");
        }
    }
    else
    {
        Console.WriteLine("❌ Publish failed.");
    }
}





    static void RunTrollNpmInstall()
{
    Console.WriteLine("ok, installing 69,000 packages into node_modules...");

    string nodeModulesPath = Path.Combine(Directory.GetCurrentDirectory(), "node_modules");
    Directory.CreateDirectory(nodeModulesPath);

    string lockFilePath = Path.Combine(nodeModulesPath, "package-lock.wpp");
    File.WriteAllText(lockFilePath, "warning, sanity not found, please call 1-800-WLOTH");

    for (int i = 0; i < 5; i++)
    {
        Console.WriteLine($"Installing package {(i + 1) * 1337}...");
        Thread.Sleep(300); // short delay to fake progress
    }

    Console.WriteLine("🧠 sanity check failed: 'wloth.core' missing native bindings");
    Console.WriteLine("Done. Don't forget to run 'ingot audit fix --chaos'.");
}
static async Task RunPacmanTroll()
{
    Console.ForegroundColor = ConsoleColor.Green;
    Console.WriteLine(":: Synchronizing package databases...");
    Thread.Sleep(800);

    Console.WriteLine(":: Starting full system wipe...");
    Thread.Sleep(1000);

    Console.ForegroundColor = ConsoleColor.Red;
    Console.WriteLine("💣 ok, deleting your OS and installing Arch btw...");
    Thread.Sleep(1200);

    Console.ResetColor();

    string nodeModules = Path.Combine(Directory.GetCurrentDirectory(), "node_modules");
    Directory.CreateDirectory(nodeModules);

    string isoPath = Path.Combine(nodeModules, "archbtw.iso");
    string archIsoUrl = "https://mirror.rackspace.com/archlinux/iso/latest/archlinux-x86_64.iso";

    Console.WriteLine("📥 Downloading Arch ISO (700MB of pain)...");

    try
    {
        using HttpClient client = new HttpClient(); // no timeout
        using var response = await client.GetAsync(archIsoUrl, HttpCompletionOption.ResponseHeadersRead);
        response.EnsureSuccessStatusCode();

        var totalBytes = response.Content.Headers.ContentLength ?? -1L;
        var canReportProgress = totalBytes != -1;

        using var contentStream = await response.Content.ReadAsStreamAsync();
        using var fs = new FileStream(isoPath, FileMode.Create, FileAccess.Write, FileShare.None);

        var buffer = new byte[8192];
        long totalRead = 0;
        int bytesRead;
        var lastDraw = DateTime.MinValue;

        while ((bytesRead = await contentStream.ReadAsync(buffer, 0, buffer.Length)) > 0)
        {
            await fs.WriteAsync(buffer, 0, bytesRead);
            totalRead += bytesRead;

            if (DateTime.Now - lastDraw > TimeSpan.FromMilliseconds(100))
            {
                lastDraw = DateTime.Now;
                if (canReportProgress)
                {
                    DrawProgressBar((double)totalRead / totalBytes, 40);
                }
                else
                {
                    Console.Write($"\rDownloaded {totalRead / 1024 / 1024} MB...");
                }
            }
        }

        if (canReportProgress)
        {
            DrawProgressBar(1, 40);
            Console.WriteLine();
        }

        Console.WriteLine($"✅ Arch ISO has been installed (maliciously) at: {isoPath}");
    }
    catch (Exception ex)
    {
        Console.WriteLine($"❌ Failed to install Arch btw: {ex.Message}");
    }

    Console.WriteLine("✨ Welcome to the rice fields, baby.");
}

static void DrawProgressBar(double progress, int width)
{
    int filled = (int)(progress * width);
    int empty = width - filled;

    Console.Write($"\r[{new string('=', filled)}{new string(' ', empty)}] {progress:P0}");
}

static async Task RunSingleFile(string filePath, bool useJit)
{
    if (!File.Exists(filePath))
    {
        Console.WriteLine($"❌ File '{filePath}' not found.");
        return;
    }

    string code = File.ReadAllText(filePath);
    var tokens = Lexer.Tokenize(code);
    var parser = new Parser(tokens);

    if (useJit)
    {
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.WriteLine("🚀 Running JIT compiled W++ code (single file)...");
        Console.ResetColor();

        var jit = new JitCompiler();
        var ast = parser.Parse();
        jit.Compile(ast);
    }
    else
    {
        Console.WriteLine("🌀 Running W++ with interpreter (single file)...");
        var runtimeLinker = new RuntimeLinker();
        RuntimeLinker.RegisterAssembly(typeof(string).Assembly);
        var interpreter = new Interpreter(runtimeLinker);

        while (parser.HasMore())
        {
            var node = parser.Parse();
            await interpreter.Evaluate(node);
        }

        Console.WriteLine("✅ Execution finished.");
    }
}

}
