# 📘 W++ Language Syntax Guide

Welcome to the official **W++ Language Syntax Guide**! W++ is a dynamic, C#-powered scripting language with first-class support for async functions, entities (OOP-style), and powerful control flow — all embedded in a clean syntax inspired by JavaScript and C-like languages.

---

## 🧠 Variables

```wpp
let x = 5;
const y = "Hello";
x = 7;
```

* Use `let` for mutable variables
* Use `const` for immutable values

---

## 🔢 Values and Types

* **Numbers**: `42`, `3.14`
* **Strings**: `"Hello, world"`
* **Booleans**: `true`, `false`
* **Null**: `null`

---

## 📏 Expressions

```wpp
let a = 1 + 2 * 3;
let b = a < 10;
let c = x ?? y; // null-coalescing
```

### Operators:

| Type          | Operators                        |   |         |
| ------------- | -------------------------------- | - | ------- |
| Arithmetic    | `+`, `-`, `*`, `/`               |   |         |
| Comparison    | `<`, `>`, `<=`, `>=`, `==`, `!=` |   |         |
| Logical       | `&&`, \`                         |   | `, `!\` |
| Null-Coalesce | `??`                             |   |         |
| Assignment    | `=`                              |   |         |

---

## 🔁 Control Flow

### If / Else

```wpp
if (x > 0) {
  print "positive";
} else {
  print "non-positive";
}
```

### While Loop

```wpp
while (x < 5) {
  x = x + 1;
}
```

### For Loop

```wpp
for (let i = 0; i < 3; i = i + 1) {
  print i;
}
```

### Switch Statement

```wpp
switch (val) {
  case 1:
    print "one";
  case 2:
    print "two";
  default:
    print "other";
}
```

### Break / Continue

```wpp
while (true) {
  if (x > 10) break;
  if (x == 5) {
    x = x + 1;
    continue;
  }
  x = x + 1;
}
```

---

## 📤 Functions and Lambdas

```wpp
let add = (x, y) => x + y;
let result = add(2, 3); // 5
```

### Async Lambda

```wpp
let fetch = async (url) => {
  return await http.get(url);
};
```

---

## 📦 Error Handling

### Try / Catch / Throw

```wpp
try {
  throw "oops";
} catch (e) {
  print "caught!";
}
```

---

## 🧬 Entities (OOP)

### Basic Entity

```wpp
entity Dog {
  speak => {
    print "Bark!";
  }
}

let myDog = new(Dog);
myDog.speak();
```

### Inheritance

```wpp
entity Husky inherits Dog {
  speak => {
    print "Woo!";
  }
}
```

### Disowning Inheritance

```wpp
entity Cat disown {
  speak => {
    print "Meow";
  }
}
```

---

## 🔄 Method Altering

```wpp
alters Husky alters Dog {
  speak => {
    print "Override";
  }
}
```

---

## 📞 Special Keywords

* `me` – refers to current entity instance
* `ancestor.methodName()` – calls method from base entity
* `new(Entity)` – create new instance
* `await` – await an async lambda/function

---

## 🛠 Imports

```wpp
import "utils.wpp";
```

---

## ✅ Printing & Returning

```wpp
print "Hello";
return 42;
```

---

## 📘 Comments

```wpp
// This is a comment
```

---

## 🧪 Example Program

```wpp
entity Helloer {
  greet => {
    print "Hello from entity!";
  }
}

let h = new(Helloer);
h.greet();
```

---

Made with ❤️ using C# and `System.Reflection.Emit`.

> W++: at least we are better then visual basic.
