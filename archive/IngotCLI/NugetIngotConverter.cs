using System.Net.Http;
using System.Reflection;
using System.Text;
using System.Xml.Linq;
using System.Text.Json;
using IngotCLI;

public static class NugetIngotConverter
{
    private static readonly HashSet<string> visited = new();

    private static List<string> ParseDependencies(string tempDir)
    {
        var nuspecPath = Directory.GetFiles(tempDir, "*.nuspec").FirstOrDefault();
        if (nuspecPath == null) return new();

        var xdoc = XDocument.Load(nuspecPath);
        var ns = xdoc.Root.GetDefaultNamespace();

        var deps = new List<string>();

        foreach (var dep in xdoc.Descendants(ns + "dependency"))
        {
            var id = dep.Attribute("id")?.Value;
            if (!string.IsNullOrEmpty(id)) deps.Add(id);
        }

        return deps;
    }
public static async Task ImportAsync(string packageName, HashSet<string> visited = null)
{
    visited ??= new();
    Console.WriteLine($"📦 Preparing NuGet package '{packageName}'...");

    if (visited.Contains(packageName))
    {
        Console.WriteLine($"🔁 Skipping already-imported: {packageName}");
        return;
    }
    visited.Add(packageName);

    string[] metaPackages =
    {
        "Microsoft.NETCore.Platforms",
        "Microsoft.NETCore.Targets",
        "System.Private.CoreLib",
        "System.Runtime",
        "System.Threading.Tasks",
        "System.Text.Encoding"
    };

    if (metaPackages.Contains(packageName))
    {
        Console.WriteLine($"🚫 Skipping meta-package: {packageName}");
        return;
    }

    var tempDir = Path.Combine(Path.GetTempPath(), "ingot_nuget_" + Guid.NewGuid());
    Directory.CreateDirectory(tempDir);

    var cacheDir = Path.Combine("PackagesCache");
    Directory.CreateDirectory(cacheDir);

    var nupkgPath = Path.Combine(cacheDir, $"{packageName}.nupkg");
    var url = $"https://www.nuget.org/api/v2/package/{packageName}";
    using var client = new HttpClient();

    if (!File.Exists(nupkgPath))
    {
        Console.WriteLine("🌐 Downloading from NuGet...");
        var bytes = await client.GetByteArrayAsync(url);
        await File.WriteAllBytesAsync(nupkgPath, bytes);
    }
    else
    {
        Console.WriteLine("📁 Using cached package.");
    }

    Console.WriteLine("📂 Extracting...");
    System.IO.Compression.ZipFile.ExtractToDirectory(nupkgPath, tempDir);

    // 📦 Parse and import dependencies
    Console.WriteLine("📦 Resolving dependencies...");
    var dependencies = ParseDependencies(tempDir);
    foreach (var dep in dependencies)
    {
        try
        {
            await ImportAsync(dep, visited); // ✅ Recursive
        }
        catch (Exception ex)
        {
            Console.WriteLine($"⚠️ Failed to import dependency '{dep}': {ex.Message}");
        }
    }

    AppDomain.CurrentDomain.AssemblyResolve += (sender, args) =>
    {
        var name = new AssemblyName(args.Name).Name + ".dll";
        var probeDirs = Directory.GetDirectories(tempDir, "*", SearchOption.AllDirectories);
        foreach (var dir in probeDirs)
        {
            var probePath = Path.Combine(dir, name);
            if (File.Exists(probePath))
                return Assembly.LoadFrom(probePath);
        }
        return null;
    };

    // 🔍 Extract useful DLLs (instead of assuming a single lib folder)
    var dlls = ExtractUsefulDlls(tempDir);
    if (dlls.Count == 0)
    {
        Console.WriteLine("❌ No usable DLLs found.");
        return;
    }

    // 📖 Optional XML documentation
    var docComments = new Dictionary<string, string>();
    var xmlPath = Path.ChangeExtension(dlls[0], ".xml");
    if (File.Exists(xmlPath))
    {
        Console.WriteLine("📖 Parsing XML doc comments...");
        var xml = XDocument.Load(xmlPath);
        foreach (var member in xml.Descendants("member"))
        {
            var nameAttr = member.Attribute("name")?.Value;
            var summary = member.Element("summary")?.Value?.Trim();
            if (!string.IsNullOrWhiteSpace(nameAttr) && !string.IsNullOrWhiteSpace(summary))
            {
                docComments[nameAttr] = summary;
            }
        }
    }

    Console.WriteLine("🔬 Reflecting DLLs...");
    var sb = new StringBuilder();
    var namespaces = new HashSet<string>();
    sb.AppendLine($"ingot {packageName} {{");

    foreach (var dll in dlls)
    {
        try
        {
            var asm = Assembly.LoadFrom(dll);
            RuntimeLinker.RegisterAssembly(asm);

            foreach (var type in asm.GetExportedTypes())
            {
                if (!type.IsClass && !type.IsInterface) continue;
                if (!string.IsNullOrWhiteSpace(type.Namespace))
                    namespaces.Add(type.Namespace);

                sb.AppendLine($"  class {type.Name} {{");

                foreach (var method in type.GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static))
                {
                    if (method.IsSpecialName) continue;

                    var returnType = method.ReturnType.Name;
                    var name = method.Name;
                    var parameters = string.Join(", ", method.GetParameters().Select(p => $"{p.ParameterType.Name} {p.Name}"));

                    string memberId = $"M:{type.FullName}.{method.Name}";
                    if (method.GetParameters().Any())
                    {
                        var paramTypes = string.Join(",", method.GetParameters().Select(p => p.ParameterType.FullName));
                        memberId += $"({paramTypes})";
                    }

                    if (docComments.TryGetValue(memberId, out var doc))
                    {
                        sb.AppendLine($"    /// {doc}");
                    }

                    sb.AppendLine($"    func {name}({parameters}): {returnType}");
                }

                sb.AppendLine("  }");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"⚠️ Skipped {Path.GetFileName(dll)}: {ex.Message}");
        }
    }

    sb.AppendLine("}");

    var usingLines = string.Join("\n", namespaces.OrderBy(n => n).Select(n => $"using {n};"));
    sb.Insert(0, usingLines + "\n\n");

    Directory.CreateDirectory("ingots");
    var targetPath = Path.Combine("ingots", packageName.ToLower() + ".ingot");
    await File.WriteAllTextAsync(targetPath, sb.ToString());

    Console.WriteLine($"✅ Ingot created: {targetPath}");

    // 🧩 Update wpp.json
    if (File.Exists("wpp.json"))
    {
        var json = File.ReadAllText("wpp.json");
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        var dependenciesJson = root.GetProperty("dependencies").EnumerateObject().ToDictionary(p => p.Name, p => p.Value.GetString());
        dependenciesJson[packageName] = "latest";

        var updated = new
        {
            name = root.GetProperty("name").GetString(),
            version = root.GetProperty("version").GetString(),
            main = root.GetProperty("main").GetString(),
            jit = root.TryGetProperty("jit", out var j) && j.GetBoolean(),
            dependencies = dependenciesJson
        };

        File.WriteAllText("wpp.json", JsonSerializer.Serialize(updated, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine("🧩 Updated wpp.json with ingot dependency.");
    }
}

       public static async Task InstallAllAsync()
{
    if (!File.Exists("wpp.json"))
    {
        Console.WriteLine("❌ No wpp.json found. Cannot install dependencies.");
        return;
    }

    var json = JsonDocument.Parse(File.ReadAllText("wpp.json"));
    var deps = json.RootElement.GetProperty("dependencies").EnumerateObject();

    foreach (var dep in deps)
    {
        var name = dep.Name;
        var ingotPath = Path.Combine("ingots", name.ToLower() + ".ingot");

        if (File.Exists(ingotPath))
        {
            Console.WriteLine($"✅ {name} already installed.");
        }
        else
        {
            Console.WriteLine($"⬇️ Installing {name}...");
            await ImportAsync(name);
        }
    }

    Console.WriteLine("🎉 All dependencies installed!");
}

public static void ListInstalled()
{
    var dir = "ingots";
    if (!Directory.Exists(dir))
    {
        Console.WriteLine("📭 No ingots installed.");
        return;
    }

    var files = Directory.GetFiles(dir, "*.ingot");
    if (files.Length == 0)
    {
        Console.WriteLine("📭 No ingots installed.");
        return;
    }

    Console.WriteLine("📦 Installed ingots:");
    foreach (var file in files)
    {
        var name = Path.GetFileNameWithoutExtension(file);
        Console.WriteLine($"  - {name}");
    }
}

    public static void RemoveIngot(string name)
    {
        var path = Path.Combine("ingots", name.ToLower() + ".ingot");
        if (!File.Exists(path))
        {
            Console.WriteLine($"❌ Ingot '{name}' not found.");
            return;
        }

        File.Delete(path);
        Console.WriteLine($"🗑️ Removed ingot '{name}'.");

        if (File.Exists("wpp.json"))
        {
            var json = File.ReadAllText("wpp.json");
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            var deps = root.GetProperty("dependencies")
                           .EnumerateObject()
                           .ToDictionary(p => p.Name, p => p.Value.GetString());

            if (deps.Remove(name))
            {
                var updated = new
                {
                    name = root.GetProperty("name").GetString(),
                    version = root.GetProperty("version").GetString(),
                    main = root.GetProperty("main").GetString(),
                    jit = root.TryGetProperty("jit", out var j) && j.GetBoolean(),
                    dependencies = deps
                };

                File.WriteAllText("wpp.json", JsonSerializer.Serialize(updated, new JsonSerializerOptions { WriteIndented = true }));
                Console.WriteLine($"🧼 Removed '{name}' from wpp.json.");
            }
        }
    }
public static List<string> ExtractUsefulDlls(string extractedPath)
{
    var usefulDlls = new List<string>();

    // Only scan inside lib/ and ignore build/, ref/, runtimes/, etc.
    var libDir = Path.Combine(extractedPath, "lib");

    if (!Directory.Exists(libDir))
        return usefulDlls;

    // Look for DLLs in target framework subfolders
    foreach (var tfmDir in Directory.GetDirectories(libDir))
    {
        var dirName = Path.GetFileName(tfmDir)?.ToLowerInvariant();

        // Skip reference-only folders
        if (dirName.StartsWith("ref") || dirName.Contains("portable") || dirName.Contains("mono"))
            continue;

        foreach (var dll in Directory.GetFiles(tfmDir, "*.dll"))
        {
            usefulDlls.Add(dll);
        }
    }

    return usefulDlls;
}


}
