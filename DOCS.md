# 🚀 W++ Partial JIT Compilation – Support Matrix

The W++ runtime supports **experimental Just-in-Time (JIT) compilation**. When using the `--jit` flag or enabling `"jit": true` in `wpp.json`, supported parts of your W++ code will be compiled to native IL instructions using the .NET JIT engine for maximum performance.

This document outlines which features are currently supported and which still fall back to the interpreter.

---

## ✅ JIT-Supported Constructs

| Feature                          | JIT Status     | Notes                                                                 |
|----------------------------------|----------------|-----------------------------------------------------------------------|
| `print "hello";`                | ✅ Supported    | Emitted using `Console.WriteLine(object)`                            |
| `let x = 5;` / `const y = 10;`  | ✅ Supported    | Stored in local IL variables                                         |
| `x = x + 1;`                    | ✅ Supported    | Assignment with arithmetic works                                     |
| `if (...) { ... } else { ... }` | ✅ Supported    | Both branches and nested ifs work                                    |
| `while (...) { ... }`          | ✅ Supported    | Full loop with condition + body + break/continue                     |
| `for (...) { ... }`            | ✅ Supported    | Supports initializer, condition, increment                           |
| `switch(x) { case ... }`       | ✅ Supported    | Case matching via `Ceq`, fallbacks to default if unmatched           |
| `break` / `continue`           | ✅ Supported    | Works inside loops and switch blocks                                 |
| `return value;`                | ✅ Supported    | Emitted using return-local and `Br` to return label                  |
| Binary operators `+ - * / < > == !=` | ✅ Supported | Proper IL opcodes mapped and boxed to object                         |
| Boolean values `true`, `false` | ✅ Supported    | Compiled as boxed `1` or `0`                                         |
| Unary `!`                      | ✅ Supported    | Translates to `Ceq` vs 0 (truth inversion)                           |
| `try/catch`                    | ✅ Supported    |                                                                      |
| `lambda` (sync only)           | ✅ Supported    |                                                                      |

---

## 🚧 Not Yet JIT-Supported (Interpreter Fallback)

| Feature                | Reason for Fallback              |
|------------------------|----------------------------------|
| `entity`, `inherits`, `disown` | Runtime object model not yet mapped to IL |
| `me`, `ancestor`, `call`       | Dynamic dispatch and method binding        |
| `async`, `await`,      | Delegate + closure handling not implemented |
| `import`                       | Runtime file evaluation stays interpreted  |


> 💡 These features will still **run correctly** when using `--jit`, but will be evaluated by the interpreter behind the scenes. No manual changes are needed.

---

## 🧪 Example: Fully JIT-Compatible Code


let x = 5;
let y = 10;

if (x < y) {
  print "x is less than y";
} else {
  print "x is greater or equal";
}

for (let i = 0; i < 5; i = i + 1) {
  print i;
}

✅ This entire snippet will be JIT compiled and run natively.

---

### 🛑 Example: Falls Back to Interpreter

entity Dog {
  speak => {
    print "Bark!";
  }
}

let mydog = new(Dog);
mydog.speak();

⚠️ entity and call operations are handled via the interpreter.

---

### 💬 Conclusion
W++ JIT mode in v0.2.2 is stable for:

Procedural logic

Arithmetic

Control flow

Printing and returning

lambdas

error handling

As the JIT evolves, more features will migrate out of the interpreter.

Stay tuned — and feel free to test JIT performance by running:


ingot run --jit
Powered by IL generation via System.Reflection.Emit.
Welcome to native W++ 🚀
