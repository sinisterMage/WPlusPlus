using System;
using System.Reflection;
using WPlusPlus;
using System.Linq;
using System.Collections.Generic;

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

        private static Type ResolveType(string typeName)
        {
            typeName = typeName.Trim('"');
            var type = Type.GetType(typeName);

            if (type != null) return type;

            foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                type = asm.GetType(typeName);
                if (type != null) return type;
            }

            throw new Exception($"❌ Type '{typeName}' not found.");
        }

        public object? Invoke(string typeName, string methodName, object[] args)
{
    Console.WriteLine($"[EXTERNCALL] \"{typeName}\".\"{methodName}\"({args.Length} arg(s))");

    string normType = typeName.Trim('"');
    string normMethod = methodName.Trim('"');

    Console.WriteLine("[EXTERNCALL DEBUG] Argument types:");
    foreach (var arg in args)
    {
        Console.WriteLine($"  - {arg} (type: {arg?.GetType()})");
    }

    // Shortcut: String.Concat
    if (normType == "System.String" && normMethod == "Concat" && args.Length == 2)
    {
        Console.WriteLine("[EXTERNCALL INFO] Using custom Concat handler");
        return string.Concat(args[0]?.ToString(), args[1]?.ToString());
    }

    try
    {
        var type = ResolveType(normType);

        // Constructor support
        if (normMethod == "ctor")
        {
            var ctor = type.GetConstructors()
                           .FirstOrDefault(c => c.GetParameters().Length == args.Length);
            if (ctor == null)
                throw new Exception($"❌ No matching constructor found on type '{typeName}' with {args.Length} parameter(s).");

            return ctor.Invoke(args);
        }

        // Instance method
        if (args.Length > 0)
        {
            var instance = args[0];
            var methodArgs = args.Skip(1).ToArray();
            var instanceType = instance?.GetType();

            if (instanceType != null)
            {
                var method = instanceType.GetMethod(normMethod, methodArgs.Select(a => a?.GetType() ?? typeof(object)).ToArray());
                if (method != null)
                {
                    Console.WriteLine($"[EXTERNCALL INFO] Matched instance method: {method}");
                    return method.Invoke(instance, methodArgs);
                }
            }
        }

        // Static method overload resolution (with typeof() fix)
        var staticMethods = type.GetMethods(BindingFlags.Public | BindingFlags.Static)
                                .Where(m => m.Name == normMethod)
                                .ToList();

        foreach (var method in staticMethods)
        {
            var parameters = method.GetParameters();
            if (parameters.Length != args.Length)
                continue;

            bool isMatch = true;
            for (int i = 0; i < parameters.Length; i++)
            {
                var expected = parameters[i].ParameterType;
                var actual = args[i];

                if (actual == null)
                {
                    if (expected.IsValueType && Nullable.GetUnderlyingType(expected) == null)
                    {
                        isMatch = false;
                        break;
                    }
                }
                else
                {
                    var actualType = actual.GetType();

                    // ✅ Special case: allow RuntimeType for expected System.Type
                    if (expected == typeof(Type) && actualType.FullName == "System.RuntimeType")
                        continue;

                    if (!expected.IsAssignableFrom(actualType))
                    {
                        isMatch = false;
                        break;
                    }
                }
            }

            if (isMatch)
            {
                Console.WriteLine($"[EXTERNCALL INFO] Matched static method: {method}");
                return method.Invoke(null, args);
            }
        }

        throw new Exception($"❌ No matching method '{normMethod}' found on type '{typeName}' with {args.Length} parameter(s).");
    }
    catch (Exception ex)
    {
        throw new Exception($"❌ Unable to invoke \"{typeName}\".\"{methodName}\" with {args.Length} arg(s)", ex);
    }
}

    }
}
