# 🦥 W++ Language Syntax Guide

*(Current Version: LLVM Backend Beta)*

> W++ — the chaotic, async-enabled, heap-happy scripting language for those who believe pointers deserve love too.
> This guide defines all syntax forms currently supported by the compiler and backend.

---

## 🗨️ Comments

Single-line comments begin with `//`.

```wpp
// This is a comment
let x = 10  // Another comment
```

Block comments are not yet supported.

---

## 🧾 Keywords

| Category           | Keywords                                                                       |
| ------------------ | ------------------------------------------------------------------------------ |
| **Control flow**   | `if`, `else`, `while`, `for`, `break`, `continue`, `switch`, `case`, `default` |
| **Declarations**   | `let`, `const`, `funcy`, `return`                                              |
| **Error handling** | `try`, `catch`, `throw`, `finally`                                             |
| **Async ops**      | `async`, `await`                                                               |
| **Booleans**       | `true`, `false`                                                                |

---

## 🆔 Identifiers

Identifiers name variables or functions.
They must start with a **letter** or **underscore**, followed by letters, digits, or underscores.

```wpp
let name = "Ofek"
let _temp = 123

funcy greet() {
    print("Hello")
}
```

---

## 🔢 Numbers

```wpp
let x = 10
let y = 255i32
let big = 100000u64
let pi = 3.14
let ratio = 2.5f64
```

Defaults:

* Integer → `i32`
* Float → `f64`

Supported suffixes: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`

---

## 🔤 Strings

Strings use **double quotes** and are **raw** — escape sequences like `\n` or `\t` are not interpreted yet.

```wpp
let s = "Hello\nWorld"
print(s) // prints literally: Hello\nWorld
```

---

## 🔣 Symbols & Operators

| Symbol            | Meaning                   |
| ----------------- | ------------------------- |
| `{ }`             | Code block                |
| `( )`             | Function call or grouping |
| `[ ]`             | Array literal             |
| `;`               | Statement separator       |
| `,`               | Argument separator        |
| `=`               | Assignment                |
| `+ - * /`         | Arithmetic                |
| `== != <= >= < >` | Comparison                |

---

## 🧩 Variables

```wpp
let name = "W++"
const version = 2
```

Heap-allocated types (arrays, objects) are stored as pointers (`i8*` internally).

---

## 🧮 Arrays

W++ supports array literals that allocate heap memory using `malloc`.

```wpp
let arr = [1, 2, 3, 4]
print(arr)
```

**How arrays work internally:**

* Stored as contiguous `i32` values in memory
* The **first slot** holds the array length
* Supported element types: integers and floats (pointers not yet supported)
* Allocated automatically on the heap

For example, `[1, 2, 3]` creates:

```
[ len=3 | 1 | 2 | 3 ]
```

---

## 🧱 Object Literals

Object literals allocate a structured heap object containing:

* a field count (`i32`)
* an array of key pointers (`i8**`)
* an array of values (`i32*`)

Example:

```wpp
let person = { "age": 30, "score": 99 }
print(person)
```

Each key is stored as a constant string in LLVM’s global data section.

Internally:

```
struct Object {
    i32 field_count;
    i8** keys;
    i32* values;
}
```

> Values default to `i32` or are converted from `f64` to `i32`.
> Future versions will support nested objects and pointer values.

---

## ⚙️ Functions

### Declaring a function

```wpp
funcy add(a, b) {
    return a + b
}
```

### Calling a function

```wpp
let result = add(3, 7)
print(result)
```

Functions return `i32` by default.

---

## 💬 Built-in Functions

### `print(value)`

Prints integers, strings, arrays, or object pointers.

```wpp
print("Hello, World!")
print(42)
print([1, 2, 3])
```

---

## 🌐 HTTP API

| Function                | Description                      |
| ----------------------- | -------------------------------- |
| `http.get(url)`         | Performs an HTTP GET request     |
| `http.post(url, body)`  | Sends a POST request             |
| `http.put(url, body)`   | Sends a PUT request              |
| `http.patch(url, body)` | Sends a PATCH request            |
| `http.delete(url)`      | Sends a DELETE request           |
| `http.status(handle)`   | Gets HTTP status code            |
| `http.body(handle)`     | Returns response body pointer    |
| `http.headers(handle)`  | Returns response headers pointer |

Example:

```wpp
let h = http.get("https://api.example.com")
print(http.status(h))
```

---

## 🧭 Server API

```wpp
funcy hello() {
    print("Hello endpoint!")
}

server.register("/hello", hello)
server.start(8080)
```

Internally uses `wpp_register_endpoint` and `wpp_start_server`.

---

## 🧠 Control Flow

### `if` / `else`

```wpp
if x > 10 {
    print("big")
} else {
    print("small")
}
```

### `while`

```wpp
let i = 0
while i < 5 {
    print(i)
    i = i + 1
}
```

### `for`

```wpp
for i = 0; i < 3; i = i + 1 {
    print("loop")
}
```

### `switch`

```wpp
switch x {
    case 1: print("one")
    case 2: print("two")
    default: print("other")
}
```

---

## 🧵 Async Functions

```wpp
async funcy fetch() {
    let res = await http.get("https://api.com")
    print(http.body(res))
}
```

*(currently requires async runtime support)*

---

## ⚠️ Error Handling

```wpp
try {
    risky_call()
} catch {
    print("oops!")
} finally {
    print("cleanup")
}
```

---

## 🧪 Example Program

```wpp
funcy main() {
    let arr = [10, 20, 30]
    let user = { "id": 1, "score": 99 }

    print(arr)
    print(user)

    server.register("/hello", hello)
    server.start(8080)
}

funcy hello() {
    print("Hello from W++!")
}
```
