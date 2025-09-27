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

## 🧠 Built-in Objects (a.k.a. “Stuff that magically works somehow”)

W++ includes several built-in objects that make it feel like a fullstack language —  
yes, you can literally make HTTP requests, JSON operations, or even spin up an API server.  
Because why not?

---

### 🌐 `http` — The Chaos Web Client

All HTTP methods are async, so don’t forget to use `await`.

```wpp
let response = await http.get("https://example.com");
print response.status;  // → 200
print response.body;

await http.post("https://api.example.com", "{ \"msg\": \"Hello\" }");
````

#### Available Methods:

| Method                             | Parameters                      | Description                  |
| ---------------------------------- | ------------------------------- | ---------------------------- |
| `http.get(url, [headers])`         | string, optional object         | Sends a GET request          |
| `http.post(url, body, [headers])`  | string, string, optional object | Sends a POST request         |
| `http.put(url, body, [headers])`   | string, string, optional object | Updates a resource           |
| `http.patch(url, body, [headers])` | string, string, optional object | Partially updates a resource |
| `http.delete(url, [headers])`      | string, optional object         | Deletes a resource           |

All methods return an object:

```wpp
{
  "status": 200,
  "body": "response body here",
  "headers": { "Content-Type": "application/json" }
}
```

---

### 🧩 `json` — JSON Without the Tears

For working with structured data, you get two helpers:

| Method                   | Description                        |
| ------------------------ | ---------------------------------- |
| `json.parse(string)`     | Converts JSON text to a W++ object |
| `json.stringify(object)` | Converts an object to JSON text    |

```wpp
let data = await json.parse("{\"hello\": \"world\"}");
print data.hello; // world

let str = await json.stringify({ "ping": "pong" });
print str; // {"ping":"pong"}
```

---

### ⚡ `api` — Make Servers Like a Madman

Yes, you can host a local HTTP API *from inside W++*.
Because who needs sanity anyway?

#### Example:

```wpp
api.start(8080);

api.endpoint("/hello", "GET", async (req, res) => {
  res.status(200);
  res.json({ message: "Hello from W++!" });
});
```

#### Available Methods:

| Method                                | Description                     |
| ------------------------------------- | ------------------------------- |
| `api.start(port)`                     | Starts the built-in HTTP server |
| `api.endpoint(path, method, handler)` | Registers a new API route       |

Handler functions receive `(req, res)` objects.

#### `req` (Request)

* `req.method` → HTTP method (GET, POST, etc.)
* `req.path` → The request path
* `req.query` → Query string
* `req.body` → Request body
* `req.headers` → Headers dictionary

#### `res` (Response)

* `res.status(code)` → Set response status
* `res.text(string)` → Send plain text response
* `res.json(object)` → Send JSON response

---

### 📝 `text` — A Debug Function That Does… Something

```wpp
text("Hello, sloths!");
```

Technically it prints stuff to the console.
Realistically, it’s just there because Ofek said so.

---



## ☕ Fun Fact

W++ might be powered by chaos, memes, and coffee,  
but it *mostly* works.  
Wait— JERRY!!! Did you actually test this build?!
