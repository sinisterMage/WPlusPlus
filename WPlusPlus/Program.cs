using System;
using System.Linq;
using System.Threading.Tasks;
using WPlusPlus;
using WPlusPlus.AST;

class Program
{
    static async Task Main(string[] args)
    {
        var code = @"
       let add = (a, b) => {
    return a + b;
};

add(2, 3);














";

        var tokens = Lexer.Tokenize(code);
        var parser = new Parser(tokens);
        var ast = parser.Parse();

        if (args.Contains("--il"))
        {
            Console.WriteLine("🚀 Running JIT compiled W++ code...");
            var jit = new JitCompiler();
            await jit.Compile(ast);
        }
        else
        {
            Console.WriteLine("🌀 Running W++ with interpreter...");
            var interpreter = new Interpreter();
            await interpreter.Evaluate(ast);
        }
    }
}
