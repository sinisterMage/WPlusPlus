# ⚡ W++ Syntax Guide (updated edtion)

> *Built with the OOPSIE™ Framework. Fueled by chaos, memes, and questionable design choices.*

---

## 📦 Variables
W++ supports `let` and `const` for declaring variables.

```wpp
let name = "Wloth";
const chaos = 9001;
````

* `let` = reassignable
* `const` = no take-backs

---

## 🔁 Control Flow

### If / Else

```wpp
if (hungry) {
  print("Eat.");
} else {
  print("Keep coding.");
}
```

### While Loop

```wpp
let i = 0;
while (i < 3) {
  print("W++ is weird");
  i = i + 1;
}
```

### For Loop

```wpp
for (let i = 0; i < 5; i = i + 1) {
  print(i);
}
```

### Switch

```wpp
switch (mood) {
  case "happy":
    print("yay!");
    break;
  case "chaotic":
    print("oh no...");
    break;
  default:
    print("neutral sloth");
}
```

### Break / Continue

```wpp
while (true) {
  if (chaos > 10) break;
  if (chaos < 0) continue;
  chaos = chaos + 1;
}
```

### Return

```wpp
let add = (x, y) => {
  return x + y;
};
```

---

## 🖨️ Print

`print` can be used with or without parentheses:

```wpp
print("Hello chaos!");
print "multiple", "args", 123;
```

---

## ⏳ Async & Await

Asynchronous code is fully supported.

```wpp
let res = await http.get("https://slothapi.dev");
print(res.status);
```

Async lambdas work too:

```wpp
let fetch = async (url) => await http.get(url);
```

---

## 🧱 Entities (OOPSIE Framework™)

Entities are W++’s version of classes — but cursed.

```wpp
entity Human {
  alters {
    speak => { print("Hello, I'm " + me.name); }
  }
}
```

* `alters` = defines methods
* `disown` = breaks inheritance
* `me` = like `this`
* `ancestor` = like `super`

### Inheritance

```wpp
entity Dog inherits Human {
  alters {
    speak => { ancestor.speak(); print("woof!"); }
  }
}
```

### Alters Outside an Entity

```wpp
Dog alters Human {
  bark => { print("woof!"); }
}
```

---

## 🧩 Object Literals

Objects can be created inline.

```wpp
let obj = {
  name: "Sloth",
  energy: 100,
  cute: true
};
```

---

## 🧪 Expressions

Dot-chaining and calls are supported:

```wpp
dev.work().sleep().repeat();
```

Assignment works too:

```wpp
me.energy = 42;
```

---

## 🛠️ Error Handling

```wpp
try {
  risky();
} catch (err) {
  print("Something broke:", err);
}
```

Throwing errors:

```wpp
throw "No coffee left!";
```

---

## 🌍 Imports

Code can be split into files and imported.

```wpp
import "utils.wpp";
```

---

## 🔌 Interop

Call external methods using `externcall`.

```wpp
externcall("System.Console", "WriteLine", ["Hello from W++"]);
```

Get type info with `typeof`.

```wpp
let t = typeof("System.String");
```

---

## 🔢 Operators

| Category   | Operators                        |   |         |
| ---------- | -------------------------------- | - | ------- |
| Arithmetic | `+`, `-`, `*`, `/`               |   |         |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` |   |         |
| Logical    | `&&`, `                          |   | `, `??` |
| Assignment | `=`                              |   |         |
| Unary      | `!`                              |   |         |
| Special    | `=>` (for methods/lambdas)       |   |         |

---

## 💀 Literals

```wpp
let yes = true;
let no = false;
let nothing = null;
let number = 123;
let text = "sloth powered";
```

---

## 🧃 Example Program

```wpp
entity Developer {
  alters {
    work => {
      me.energy = me.energy - 10;
      if (me.energy <= 0) {
        return "Out of energy!";
      }
      print("Energy left:", me.energy);
    }
  }
}

let dev = new Developer();
dev.work();
```

---

## ☕ Fun Fact

W++ might be powered by chaos, memes, and coffee,  
but it *mostly* works.  
Wait— JERRY!!! Did you actually test this build?!
