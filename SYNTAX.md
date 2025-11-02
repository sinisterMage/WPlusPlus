# 🦥 W++ Language Syntax Guide

*(W++ v2.0 - LLVM Backend)*

> W++ — the chaotic, LLVM-compiled, multi-paradigm programming language where sloths meet native performance.
> This guide defines all syntax forms currently supported by the W++ v2 compiler.

---

## Table of Contents

- [Comments](#-comments)
- [Keywords](#-keywords)
- [Identifiers](#-identifiers)
- [Types](#-types)
  - [Primitives](#primitives)
  - [Type Aliases](#type-aliases)
  - [Entities](#entities-oop)
- [Literals](#-literals)
- [Variables](#-variables)
- [Operators](#-operators)
- [Data Structures](#-data-structures)
  - [Arrays](#arrays)
  - [Objects](#objects)
- [Functions](#-functions)
- [Control Flow](#-control-flow)
- [Multiple Dispatch](#-multiple-dispatch)
- [Entities & OOP](#-entities--oop)
- [String Operations](#-string-operations)
- [Threading](#-threading)
- [Async/Await](#-asyncawait)
- [HTTP API](#-http-api)
- [Server API](#-server-api)
- [Module System](#-module-system)
- [Error Handling](#-error-handling)
- [Built-in Functions](#-built-in-functions)
- [Example Programs](#-example-programs)

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
| **Declarations**   | `let`, `const`, `funcy`, `func`, `return`                                      |
| **Error handling** | `try`, `catch`, `throw`, `finally`                                             |
| **Async ops**      | `async`, `await`                                                               |
| **Booleans**       | `true`, `false`                                                                |
| **OOP**            | `entity`, `alters`, `new`, `me`                                                |
| **Modules**        | `import`, `export`, `from`, `type`                                             |

---

## 🆔 Identifiers

Identifiers name variables, functions, entities, and types.
They must start with a **letter** or **underscore**, followed by letters, digits, or underscores.

```wpp
let name = "Ofek"
let _temp = 123
let myFunction = add

funcy greet() {
    print("Hello")
}

entity Dog {
    bark => { print("Woof!") }
}
```

---

## 🎯 Types

W++ uses **static typing** with **type inference**.

### Primitives

| Type   | Description          | Default For  |
|--------|----------------------|--------------|
| `i8`   | 8-bit signed int     |              |
| `i16`  | 16-bit signed int    |              |
| `i32`  | 32-bit signed int    | integers     |
| `i64`  | 64-bit signed int    |              |
| `u8`   | 8-bit unsigned int   |              |
| `u16`  | 16-bit unsigned int  |              |
| `u32`  | 32-bit unsigned int  |              |
| `u64`  | 64-bit unsigned int  |              |
| `f32`  | 32-bit float         |              |
| `f64`  | 64-bit float         | floats       |
| `bool` | Boolean (true/false) |              |
| `str`  | String (pointer)     | string literals |

### Type Aliases

Define named object types:

```wpp
type PersonObject {
    "name": str,
    "age": i32
}

funcy greet(person: PersonObject) {
    print("Hello", person.name)
}
```

### Entities (OOP)

Define custom types with methods and inheritance:

```wpp
entity Animal {
    name => "Unknown"
    speak => { print("...") }
}

entity Dog alters Animal {
    speak => { print("Woof!") }
}

let mydog = new(Dog)
mydog.speak()  // prints: Woof!
```

---

## 💎 Literals

### Numbers

```wpp
let x = 10          // i32 by default
let y = 255i32      // explicit i32
let big = 100000u64 // unsigned 64-bit
let pi = 3.14       // f64 by default
let ratio = 2.5f64  // explicit f64
```

**Supported suffixes**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`

### Strings

Strings use **double quotes** and support escape sequences.

```wpp
let s = "Hello\nWorld"
print(s)  // prints: Hello
          //         World

let greeting = "Hello\tWorld"
print(greeting)  // prints: Hello    World

let quote = "She said \"Hello\""
print(quote)  // prints: She said "Hello"
```

**Supported Escape Sequences**:

| Escape | Description | Example |
|--------|-------------|---------|
| `\n` | Newline (line feed) | `"line1\nline2"` |
| `\t` | Horizontal tab | `"col1\tcol2"` |
| `\r` | Carriage return | `"text\rmore"` |
| `\\` | Backslash | `"path\\to\\file"` |
| `\"` | Double quote | `"Say \"hi\""` |
| `\'` | Single quote | `"It\'s nice"` |

### Booleans

```wpp
let isReady = true
let isDone = false
```

### Arrays

```wpp
let numbers = [1, 2, 3, 4]
let floats = [1.1, 2.2, 3.3]
```

### Objects

```wpp
let person = { "name": "Ofek", "age": 30 }
let config = { "port": 8080, "debug": 1 }
```

---

## 🧩 Variables

```wpp
let name = "W++"     // mutable variable
const version = 2    // immutable constant
```

**Notes**:
- Variables default to `i32` for integers, `f64` for floats
- Heap-allocated types (arrays, objects, strings) are stored as pointers
- Type inference determines type from value

---

## 🔣 Operators

### Arithmetic

```wpp
let sum = a + b
let diff = a - b
let product = a * b
let quotient = a / b
```

### Comparison

```wpp
if (x == y) { }    // equality
if (x != y) { }    // inequality
if (x < y) { }     // less than
if (x > y) { }     // greater than
if (x <= y) { }    // less than or equal
if (x >= y) { }    // greater than or equal
```

### String Operations

```wpp
// String comparison
if (name == "Ofek") { }
if (path != "/home") { }

// String concatenation
let full = first + " " + last

// String length
let len = strlen(name)

// Integer to string
let numStr = int_to_string(42)
```

### Logical

```wpp
if (!isReady) { }           // logical NOT
if (x > 0 && y > 0) { }     // logical AND (via nested ifs)
if (x > 0 || y > 0) { }     // logical OR (via else branch)
```

---

## 📦 Data Structures

### Arrays

Arrays allocate heap memory and store a length prefix.

```wpp
let arr = [1, 2, 3, 4]
print(arr)  // prints: [1, 2, 3, 4]
```

**Internal representation**:
```
[ length | elem1 | elem2 | elem3 | ... ]
```

**Supported element types**: integers, floats
**Future**: pointer elements, nested arrays

### Objects

Objects are structured heap allocations with key-value pairs.

```wpp
let user = { "id": 1, "score": 99 }
print(user)  // prints object structure
```

**Internal representation**:
```c
struct Object {
    i32 field_count;
    i8** keys;         // array of string pointers
    i32* values;       // array of i32 values
}
```

Keys are stored as global constants in LLVM IR.

---

## ⚙️ Functions

### Declaration

```wpp
funcy add(a, b) {
    return a + b
}

// Alternative syntax
func multiply(x, y) {
    return x * y
}
```

### Calling

```wpp
let result = add(3, 7)
print(result)  // 10
```

### Return Values

Functions return `i32` by default. Explicit return types are inferred from return statements.

```wpp
funcy getPI() {
    return 3.14159f64  // returns f64
}
```

### Implicit Returns

The last expression in a function body is implicitly returned:

```wpp
funcy double(x) {
    x * 2  // implicitly returned
}
```

---

## 🧠 Control Flow

### If / Else

```wpp
if (x > 10) {
    print("big")
} else {
    print("small")
}

// Nested conditions
if (x > 10) {
    if (x < 20) {
        print("medium")
    }
}
```

### While Loops

```wpp
let i = 0
while (i < 5) {
    print(i)
    i = i + 1
}
```

### For Loops

```wpp
for (i = 0; i < 5; i = i + 1) {
    print(i)
}

// With explicit let
for (let j = 0; j < 3; j = j + 1) {
    print("loop", j)
}
```

### Switch / Case

```wpp
switch (x) {
    case 1: print("one")
    case 2: print("two")
    case 3: print("three")
    default: print("other")
}
```

### Break / Continue

```wpp
while (true) {
    if (done) {
        break    // exit loop
    }
    if (skip) {
        continue  // next iteration
    }
    // work...
}
```

---

## 🎯 Multiple Dispatch

W++ supports **multiple dispatch** — functions can have different implementations based on:
- Parameter types
- HTTP status codes
- Entity types
- Object types

### Type-Based Dispatch

```wpp
// Generic function
funcy process(data) {
    print("Processing generic data")
}

// Specialized for specific object type
funcy process(user: UserObject) {
    print("Processing user:", user.name)
}

// Specialized for HTTP 2xx responses
funcy handle(response: 2xx) {
    print("Success!")
}

// Specialized for HTTP 404
funcy handle(response: 404) {
    print("Not found")
}
```

### Dispatch Specificity

When multiple implementations match, the most specific one wins:

1. **HTTP Status Literal** (e.g., `404`) - specificity: 100
2. **Object Type / Entity** (e.g., `UserObject`) - specificity: 90
3. **Primitive Type** (e.g., `i32`) - specificity: 80
4. **Function Type** (e.g., `func(i32) -> i32`) - specificity: 60 + parameter/return specificity
5. **HTTP Status Range** (e.g., `2xx`) - specificity: 50
6. **Any** (no type annotation) - specificity: 0

### Higher-Order Dispatch

W++ supports **higher-order dispatch** — functions can dispatch on the signatures of function-typed parameters using the `func(...)` type annotation syntax:

```wpp
// Dispatch on function parameter types
funcy apply(fn: func(i32) -> i32, data: i32) -> i32 {
    print("Applying integer function")
    return fn(data)
}

funcy apply(fn: func(string) -> string, data: string) -> string {
    print("Applying string function")
    return fn(data)
}

// Helper functions
func double(x: i32) -> i32 {
    return x * 2
}

func uppercase(s: string) -> string {
    return s  // placeholder
}

// Calls dispatch to first overload based on function signature
let result1 = apply(double, 5)  // prints "Applying integer function", returns 10

// Calls dispatch to second overload
let result2 = apply(uppercase, "hello")  // prints "Applying string function"
```

#### Function Type Syntax

Function types use the syntax: `func(ParamType1, ParamType2, ...) -> ReturnType`

- **Parameters**: Comma-separated list of parameter types
- **Return Type**: Optional, specified with `-> Type` (defaults to `i32` if omitted)
- **Empty Parameters**: Use `func() -> RetType` for functions with no parameters

Examples:
```wpp
func(i32) -> i32              // Single parameter, returns i32
func(i32, string) -> bool     // Two parameters, returns bool
func() -> i32                 // No parameters, returns i32
func(i32, i32)                // Two parameters, returns i32 (default)
```

#### Function Type Specificity

Function types have a base specificity of 60, plus additional specificity based on their parameter and return types. This means:
- A function type is more specific than HTTP status ranges
- A function type is less specific than primitive types when used alone
- The total specificity increases with more specific parameter and return types

---

## 🏛️ Entities & OOP

Entities provide object-oriented programming features.

### Defining Entities

```wpp
entity Animal {
    name => "Unknown"
    age => 0

    speak => {
        print("...")
    }
}
```

### Inheritance

```wpp
entity Dog alters Animal {
    breed => "Mixed"

    speak => {
        print("Woof! I'm", me.name)
    }
}

entity Cat alters Animal {
    speak => {
        print("Meow")
    }
}
```

### Creating Instances

```wpp
let mydog = new(Dog)
let mycat = new(Cat)

mydog.speak()  // Woof! I'm Unknown
mycat.speak()  // Meow
```

### The `me` Keyword

Inside entity methods, `me` refers to the current instance:

```wpp
entity Counter {
    value => 0

    increment => {
        me.value = me.value + 1
    }
}
```

---

## 🔤 String Operations

W++ v2 provides native string operations.

### String Comparison

```wpp
if (name == "Ofek") {
    print("Hello Ofek!")
}

if (path != "/home") {
    print("Not home directory")
}
```

Uses `strcmp` internally for proper string comparison.

### String Concatenation

```wpp
let first = "Hello"
let second = " World"
let greeting = first + second  // "Hello World"

// Building messages
let name = "Alice"
let age = int_to_string(30)
let message = "Name: " + name + ", Age: " + age
```

**Note**: Deep nesting may cause issues. Use intermediate variables for complex concatenations.

### String Length

```wpp
let text = "Hello"
let len = strlen(text)  // 5
print("Length:", len)
```

### Integer to String Conversion

```wpp
let num = 42
let str = int_to_string(num)  // "42"
print("The answer is " + str)
```

---

## 🧵 Threading

W++ provides garbage-collected thread management with auto-join.

### Spawning Threads

```wpp
funcy worker() {
    print("Running in thread")
    return 0
}

// Blocking mode (default) - waits for completion
let t1 = useThread(worker)

// Detached mode - runs concurrently
let t2 = useThread(worker, 1)
```

### Thread Modes

**Blocking (default)**:
```wpp
let t = useThread(worker)
// Main thread blocks here until worker completes
print("Worker finished")
```

**Detached (concurrent)**:
```wpp
let t = useThread(worker, 1)
// Main thread continues immediately
print("Worker running in background")
// Threads auto-join on program exit
```

### Thread-Safe State

```wpp
let counter = useThreadState(0)

funcy increment() {
    let current = wpp_thread_state_get(counter)
    wpp_thread_state_set(counter, current + 1)
}

let t1 = useThread(increment, 1)
let t2 = useThread(increment, 1)
// counter will be safely incremented by both threads
```

### Auto-Join

All threads automatically join when:
- The thread handle goes out of scope
- The program exits
- Manual join via `wpp_thread_join()` (advanced)

**No manual thread management needed!**

---

## ⚡ Async/Await

W++ supports async functions with tokio-backed runtime.

### Async Functions

```wpp
async funcy fetchData(url) {
    let response = await http.get(url)
    return http.body(response)
}
```

### Await Expressions

```wpp
async funcy processAPI() {
    let data = await fetchData("https://api.example.com")
    print(data)
}
```

### Async with HTTP

```wpp
async funcy handleRequest() {
    let response = await http.post("https://api.com/data", body)
    let status = http.status(response)

    if (status == 200) {
        print("Success!")
    }
}
```

---

## 🌐 HTTP API

Built-in HTTP client using `reqwest` (async).

### GET Request

```wpp
let response = http.get("https://api.github.com/users/sinisterMage")
let status = http.status(response)
let body = http.body(response)

print("Status:", status)
print("Body:", body)
```

### POST Request

```wpp
let data = "{ \"name\": \"Ofek\" }"
let response = http.post("https://api.example.com/users", data)
```

### Other Methods

```wpp
// PUT
let response = http.put("https://api.com/resource/1", updateData)

// PATCH
let response = http.patch("https://api.com/resource/1", patchData)

// DELETE
let response = http.delete("https://api.com/resource/1")
```

### Response Handling

```wpp
let response = http.get(url)

// Status code (i32)
let status = http.status(response)

// Response body (string pointer)
let body = http.body(response)

// Response headers (string pointer)
let headers = http.headers(response)
```

---

## 🧭 Server API

Built-in HTTP server using tokio async runtime.

### Registering Endpoints

```wpp
funcy hello() {
    print("Hello endpoint called!")
    return 0
}

funcy getUser() {
    print("Fetching user data")
    return 0
}

server.register("/hello", hello)
server.register("/user", getUser)
```

### Starting Server

```wpp
funcy main() {
    server.register("/api/status", checkStatus)
    server.register("/api/health", healthCheck)

    print("Starting server on port 8080...")
    server.start(8080)
}
```

Server runs asynchronously and handles concurrent connections.

---

## 📚 Module System

W++ supports importing and exporting code across files.

### Exporting

```wpp
// math.wpp
export funcy add(a, b) {
    return a + b
}

export funcy multiply(a, b) {
    return a * b
}
```

### Importing

```wpp
// main.wpp
import "math.wpp"

funcy main() {
    let sum = add(5, 3)
    let product = multiply(4, 7)
    print(sum, product)
}
```

### Import Specific Items

```wpp
from "math.wpp" import add, multiply

funcy main() {
    print(add(1, 2))
}
```

### Rust Module Integration

```wpp
// Import compiled Rust modules
import "rust:json"
import "rust:io"

funcy main() {
    let data = json_parse("{ \"key\": \"value\" }")
    io_write_file("output.txt", data)
}
```

Rust modules are dynamically linked and provide FFI functions to W++.

---

## ⚠️ Error Handling

### Try / Catch / Finally

```wpp
try {
    risky_operation()
} catch {
    print("Error occurred!")
} finally {
    print("Cleanup code runs regardless")
}
```

### Throwing Errors

```wpp
funcy validateAge(age) {
    if (age < 0) {
        throw("Age cannot be negative")
    }
    return true
}

try {
    validateAge(-5)
} catch {
    print("Validation failed")
}
```

---

## 💬 Built-in Functions

### `print(...)`

Prints values to stdout. Supports multiple arguments.

```wpp
print("Hello")                    // string
print(42)                         // integer
print(3.14)                       // float
print(true)                       // boolean
print([1, 2, 3])                  // array
print({ "key": "value" })         // object

// Multiple arguments
print("Sum:", 10 + 20)
print("User:", name, "Age:", age)
```

### `readline()`

Reads a line from stdin.

```wpp
let input = readline()
print("You entered:", input)
```

### String Functions

```wpp
strlen(str)           // Get string length
int_to_string(num)    // Convert integer to string
```

### Thread Functions

```wpp
useThread(func)              // Spawn blocking thread
useThread(func, 1)           // Spawn detached thread
useThreadState(initial)      // Create thread-safe state
```

---

## 🧪 Example Programs

### Hello World

```wpp
funcy main() {
    print("Hello, W++ v2!")
    return 0
}
```

### HTTP Client

```wpp
funcy main() {
    let response = http.get("https://api.github.com")
    let status = http.status(response)

    if (status == 200) {
        let body = http.body(response)
        print("Success! Body:", body)
    } else {
        print("Request failed with status:", status)
    }
}
```

### Web Server

```wpp
funcy handleRequest() {
    print("Request received")
    return 0
}

funcy main() {
    server.register("/api", handleRequest)
    print("Server starting on port 8080")
    server.start(8080)
}
```

### Multi-Threading

```wpp
funcy worker(id) {
    print("Worker", id, "running")
    let i = 0
    while (i < 1000000) {
        i = i + 1
    }
    print("Worker", id, "done")
    return 0
}

funcy main() {
    print("Spawning 4 concurrent workers")

    let t1 = useThread(worker, 1)  // detached
    let t2 = useThread(worker, 1)
    let t3 = useThread(worker, 1)
    let t4 = useThread(worker, 1)

    print("All workers spawned")
    print("Main thread continues")

    return 0  // Threads auto-join on exit
}
```

### Entity-Based OOP

```wpp
entity Animal {
    name => "Unknown"
    speak => { print("...") }
}

entity Dog alters Animal {
    breed => "Mixed"
    speak => { print("Woof! I'm", me.name) }
}

entity Cat alters Animal {
    speak => { print("Meow") }
}

funcy main() {
    let dog = new(Dog)
    let cat = new(Cat)

    dog.speak()  // Woof! I'm Unknown
    cat.speak()  // Meow
}
```

### Multiple Dispatch

```wpp
// Generic handler
funcy handle(data) {
    print("Handling generic data")
}

// Specialized for HTTP success
funcy handle(response: 2xx) {
    print("Success! Status:", http.status(response))
}

// Specialized for HTTP 404
funcy handle(response: 404) {
    print("Resource not found")
}

funcy main() {
    let response = http.get("https://api.com")
    handle(response)  // Calls most specific handler
}
```

---



**W++ v2 - Powered by LLVM and chaos 🦥🚀**
