using System.Net.Http;
using System.Reflection;
using System.Text;
using System.Xml.Linq;
using System.Text.Json;

public static class NugetIngotConverter
{
    public static async Task ImportAsync(string packageName)
    {
        Console.WriteLine($"📦 Downloading NuGet package '{packageName}'...");

        var tempDir = Path.Combine(Path.GetTempPath(), "ingot_nuget_" + Guid.NewGuid());
        Directory.CreateDirectory(tempDir);

        var nupkgPath = Path.Combine(tempDir, $"{packageName}.nupkg");
        using var client = new HttpClient();
        var url = $"https://www.nuget.org/api/v2/package/{packageName}";

        var bytes = await client.GetByteArrayAsync(url);
        await File.WriteAllBytesAsync(nupkgPath, bytes);

        Console.WriteLine("📂 Extracting...");
        System.IO.Compression.ZipFile.ExtractToDirectory(nupkgPath, tempDir);

        var libDirs = Directory.GetDirectories(Path.Combine(tempDir, "lib"));
        var libDir = libDirs.FirstOrDefault(d => d.Contains("net")) ?? libDirs.FirstOrDefault();

        if (libDir is null)
        {
            Console.WriteLine("❌ No library folder found in NuGet package.");
            return;
        }

        var dll = Directory.GetFiles(libDir, "*.dll").FirstOrDefault();
        if (dll is null)
        {
            Console.WriteLine("❌ No usable DLL found.");
            return;
        }

        var xmlPath = Path.ChangeExtension(dll, ".xml");
        var docComments = new Dictionary<string, string>();

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

        Console.WriteLine("🔬 Reflecting DLL...");
        var sb = new StringBuilder();
        sb.AppendLine($"ingot {packageName} {{");

        var asm = Assembly.LoadFile(dll);
        foreach (var type in asm.GetExportedTypes())
        {
            if (!type.IsClass && !type.IsInterface) continue;

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

        sb.AppendLine("}");

        Directory.CreateDirectory("ingots");
        var targetPath = Path.Combine("ingots", packageName.ToLower() + ".ingot");
        await File.WriteAllTextAsync(targetPath, sb.ToString());

        Console.WriteLine($"✅ Ingot created: {targetPath}");

        // ✅ Update wpp.json if exists
        if (File.Exists("wpp.json"))
        {
            var json = File.ReadAllText("wpp.json");
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            var dependencies = root.GetProperty("dependencies").EnumerateObject().ToDictionary(p => p.Name, p => p.Value.GetString());
            dependencies[packageName] = "latest";

            var updated = new
            {
                name = root.GetProperty("name").GetString(),
                version = root.GetProperty("version").GetString(),
                main = root.GetProperty("main").GetString(),
                jit = root.TryGetProperty("jit", out var j) && j.GetBoolean(),
                dependencies = dependencies
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

}
