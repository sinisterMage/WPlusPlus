using System;
using System.IO;
using WPlusPlus;
using WPlusPlus.Shared;
using IngotCLI;

class Program
{
    static async Task Main(string[] args)
    {
        string bundlePath = Path.Combine(AppContext.BaseDirectory, "bundle.wpp");
        if (!File.Exists(bundlePath))
        {
            Console.WriteLine("❌ bundle.wpp not found in runner directory.");
            return;
        }

        var code = File.ReadAllText(bundlePath);
        var tokens = Lexer.Tokenize(code);
        var parser = new Parser(tokens);

        var runtimeLinker = new RuntimeLinker();
        RuntimeLinker.RegisterAssembly(typeof(string).Assembly);
        var interpreter = new Interpreter(runtimeLinker);

        while (parser.HasMore())
        {
            var node = parser.Parse();
            await interpreter.Evaluate(node);
        }
    }
}
