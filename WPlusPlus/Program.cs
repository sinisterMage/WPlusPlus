/*using System;
using System.Threading.Tasks;
using WPlusPlus;
using WPlusPlus.AST;

class Program
{
    static async Task Main(string[] args)
    {
        string source = @"
    let x = 2;

switch (x) {
    case 1:
        print ""one"";
        break;

    case 2:
        print ""two"";
        break;

    case 3:
        print ""three"";
        break;

    default:
        print ""unknown"";
}

    ";

        var tokens = Lexer.Tokenize(source);

        // 🔍 Dump all tokens for debugging
        Console.WriteLine("=== TOKEN STREAM ===");
        foreach (var token in tokens)
        {
            Console.WriteLine($"Type: {token.Type}, Value: '{token.Value}'");
        }
        Console.WriteLine("====================\n");

        var parser = new Parser(tokens);
        var interpreter = new Interpreter();

        while (parser.HasMore())
        {
            var node = parser.Parse();
            await interpreter.Evaluate(node);
        }
    }

}*/
