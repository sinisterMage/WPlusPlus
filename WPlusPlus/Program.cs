using System;
using System.Threading.Tasks;
using WPlusPlus;
using WPlusPlus.AST;

class Program
{
    static async Task Main(string[] args)
    {
        var code = @"
// === Variables and Math ===
let x = 5;
let y = 10;
print(x + y);             // 15
print(x * 2);             // 10
print((x + y) / 3);       // 5

// === Conditionals ===
if (x < y) {
    print(1);             // 1
} else {
    print(0);
}

// === Loops ===
let i = 0;
while (i < 3) {
    print(i);
    i = i + 1;
}

// === Functions ===
let add = (a, b) => {
    return a + b;
};
print(add(7, 8));         // 15

// === Async Functions ===
let asyncAdd = async(a, b) => {
    return a + b;
};
await asyncAdd(2, 3);     // (prints 5 if you choose to log)

// === Try/Catch ===
try {
    throw ""error!"";
} catch (e) {
    print(999);           // 999
}

// === Entities ===
entity Animal {
    speak => {
        print(""Animal speaks"");
    }
    whoami => {
        print(""I am an animal"");
    }
}

entity Dog inherits Animal {
    speak => {
        print(""Dog barks"");
    }
}

entity Cat inherits Animal {
    speak => {
        print(""Cat meows"");
    }
}

alters Cat alters Animal {
    whoami => {
        print(""I am a cat"");
        ancestor.whoami();
    }
}

let dog = new(Dog);
let cat = new(Cat);

print(""=== Entity Tests ==="");
dog.speak();             // Dog barks
dog.whoami();            // I am an animal
cat.speak();             // Cat meows
cat.whoami();            // I am a cat, I am an animal

// === me keyword ===
entity Selfie {
    hello => {
        if (me != null) {
            print(""me is valid"");
        }
    }
}

let s = new(Selfie);
s.hello();               // me is valid";

        var tokens = Lexer.Tokenize(code);
        foreach (var t in tokens)
{
    Console.WriteLine($"[{t.Type}] {t.Value}");
}

        var parser = new Parser(tokens);
        var ast = parser.Parse();

        var interpreter = new Interpreter();
        await interpreter.Evaluate(ast);
    }
}
