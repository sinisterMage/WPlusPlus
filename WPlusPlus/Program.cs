using System;
using System.Linq;
using System.Threading.Tasks;
using IngotCLI;
using WPlusPlus;
using WPlusPlus.AST;
using WPlusPlus.Shared;

class Program
{
    static async Task Main(string[] args)
    {
        var code = @"
       // Prepare request bodies and headers
let putBody = json.stringify({
  name: ""Wloth""
});

let patchBody = json.stringify({
  mood: ""chaotic""
});

let jsonHeaders = {
  ""Content-Type"": ""application/json""
};

let deleteHeaders = {
  ""X-Delete-By"": ""W++""
};

// PUT
let putRes = await http.put(""https://httpbin.org/put"", putBody, jsonHeaders);
print(""PUT status: "");
print(putRes.status);
print(""\nPUT body: "");
print(putRes.body);
print(""\n"");

// PATCH
let patchRes = await http.patch(""https://httpbin.org/patch"", patchBody, jsonHeaders);
print(""PATCH status: "");
print(patchRes.status);
print(""\nPATCH body: "");
print(patchRes.body);
print(""\n"");

// DELETE
let deleteRes = await http.delete(""https://httpbin.org/delete"", deleteHeaders);
print(""DELETE status: "");
print(deleteRes.status);
print(""\nDELETE body: "");
print(deleteRes.body);
print(""\n"");






















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
            var runtimeLinker = new RuntimeLinker();
            var interpreter = new Interpreter(runtimeLinker);
            await interpreter.Evaluate(ast);
        }
    }
}
