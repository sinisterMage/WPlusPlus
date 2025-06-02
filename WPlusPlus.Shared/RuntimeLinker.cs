using System.Reflection;
using WPlusPlus;
using System.Linq;

namespace IngotCLI
{
    public class RuntimeLinker : IRuntimeLinker
    {
        private static readonly List<Assembly> loadedAssemblies = new();

        public static void RegisterAssembly(Assembly asm)
        {
            if (!loadedAssemblies.Contains(asm))
                loadedAssemblies.Add(asm);
        }

        public object? Invoke(string typeName, string methodName, object[] args)
{
    Console.WriteLine($"[EXTERNCALL] \"{typeName}\".\"{methodName}\"({args.Length} arg(s))");

    // Normalize the names in case quotes are included
    string normType = typeName.Trim('"');
    string normMethod = methodName.Trim('"');

    // Debug argument types
    Console.WriteLine("[EXTERNCALL DEBUG] Argument types:");
    foreach (var arg in args)
    {
        Console.WriteLine($"  - {arg} (type: {arg?.GetType()})");
    }

    // Custom shortcut for string concatenation
    if (normType == "System.String" && normMethod == "Concat" && args.Length == 2)
    {
        Console.WriteLine("[EXTERNCALL INFO] Using custom Concat handler");
        return string.Concat(args[0]?.ToString(), args[1]?.ToString());
    }

    try
    {
        var type = Type.GetType(normType);
        if (type == null)
        {
            throw new Exception($"❌ Type '{normType}' not found.");
        }

        var methods = type.GetMethods().Where(m => m.Name == normMethod).ToList();
        foreach (var method in methods)
        {
            var parameters = method.GetParameters();
            if (parameters.Length == args.Length)
            {
                return method.Invoke(null, args);
            }
        }

        throw new Exception($"❌ No matching method '{normMethod}' found on type '{normType}' with {args.Length} parameter(s).");
    }
    catch (Exception ex)
    {
        throw new Exception($"❌ Unable to invoke \"{typeName}\".\"{methodName}\" with {args.Length} arg(s)", ex);
    }
}



    }
}
