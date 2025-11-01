# 🦥 W++ Language Support for VS Code

Complete language support for the **W++** programming language with **Language Server Protocol (LSP)** integration.

> Powered by chaos. Driven by sloths. Built for web development and concurrent applications.

---

## ✨ Features

### 🎨 Syntax Highlighting
- Full syntax highlighting for W++ v2
- 31 keywords including `funcy`, `try/catch`, `type`, `export`
- Support for entities, type aliases, and async/await

### 🧠 Intelligent Code Completion
- Context-aware keyword suggestions
- Snippet-based completions with placeholders
- Smart suggestions for:
  - Exception handling (`try`/`catch`/`finally`)
  - Function declarations (`funcy`)
  - Module imports/exports
  - Type aliases and entities

### 🔍 Real-time Error Diagnostics
- Parse errors detected as you type
- Integration with `ingot` compiler
- Error messages appear in Problems panel
- 500ms debounce for smooth editing

### 📝 Code Snippets
- 20+ code snippets for common patterns
- Function declarations with type annotations
- Entity definitions with inheritance
- Exception handling blocks
- Import/export statements

### ⚡ Quick Actions
- **Run W++ File**: Execute current file with `ingot run`
- File icons for `.wpp` files
- Automatic syntax detection

---

## ⚙️ Requirements

To run W++ files, install the **Ingot CLI** — the official W++ toolchain.

### 🛠️ Installing W++ (v0.2.6)

1. Go to the official repo: [github.com/sinisterMage/WPlusPlus](https://github.com/sinisterMage/WPlusPlus)
2. Click the ⏬ **Releases** tab
3. Choose your path:
   - 🅰️ Download the installer for your OS *(FreeBSD support coming soon™)*
   - 🅱️ Download `ingot` directly and add it to your `PATH`
   - 🆎 Build from source — if you enjoy pain

4. Run these commands:
   ```bash
   ingot init   # Create a new W++ project
   ingot run    # Run it
   ingot help   # Behold the divine scroll of commands
    ```

---

## ▶️ Running W++ Code in VS Code

1. Open any `.wpp` file
2. Press `Ctrl+Shift+P`
3. Select **“Run W++ File with Ingot”**
4. Output will appear in the terminal

---

## 🧠 Sample W++ Code

```wpp
// Function with type annotations
funcy greet(name: string) -> string {
    return "Hello, " + name
}

// Exception handling
try {
    let result = greet("World")
    print(result)
} catch (e) {
    print("Error:", e)
}

// Entity with inheritance
entity Animal {
    name: string
}

entity Dog alters Animal {
    breed: string
}

// Type alias
type User = {
    name: string,
    age: int
}
```

---

## 🔧 LSP Features Setup

The extension includes a Language Server that provides smart features:

### Automatic Setup
- LSP server starts automatically when you open a `.wpp` file
- No configuration needed!

### Error Diagnostics
- Requires `ingot` CLI to be installed and in PATH
- Errors appear in the Problems panel (Ctrl+Shift+M)
- Updates automatically as you type (500ms debounce)

### Code Completion
- Press `Ctrl+Space` for suggestions
- Works in any `.wpp` file
- Context-aware (e.g., suggests `catch` after `try`)

### Troubleshooting
If LSP features aren't working:
1. Check that `ingot` is installed: `ingot --version`
2. Reload VSCode: `Ctrl+Shift+P` → "Reload Window"
3. Check Output panel: `View` → `Output` → Select "W++ Language Server"

---

## 📁 File Association

* Files with `.wpp` extension are automatically recognized

---

## 🔗 Links

* 🌐 Official Website: [wplusplus.org](https://wplusplus.org)
* 📦 GitHub Repo: [sinisterMage/WPlusPlus](https://github.com/sinisterMage/WPlusPlus)

---

## 🤝 Contributing

Want to improve this extension or add new features?

Email me: **[ofek@wplusplus.org](mailto:ofek@wplusplus.org)**

PRs, issues, and chaotic ideas are welcome.

---

## 📜 License

MIT License
Built with ❤️ for creative coders, language tinkerers, and curious minds.


