# W++ 🦥  ![image](https://github.com/user-attachments/assets/e55dc88e-7ef0-4aa6-8d3e-fbb77c9aac08)
![W++ Extension: Resurrected](https://img.shields.io/badge/W%2B%2B%20Extension-Resurrected-informational?style=flat-square&color=purple&logo=github&logoColor=white)

> *At least we’re better than Visual Basic.*

**W++** is a fun, experimental, and completely over-engineered programming language designed for learning, chaos, and memes.  
It includes async lambdas, pseudo-OOPSIE principles (Object-Oriented Programming Sometimes Isn’t Excellent), and full integration with a custom-built VSCode extension.

This repo contains the full source code of W++ after it reached over **33,000 downloads** on the VSCode Marketplace — and was mysteriously flagged and removed.

---



## 📰 Extension Reinstated After Legal Pushback

After some back-and-forth with Microsoft — and thanks to **external pressure** and the help of a **very persistent lawyer (hi mom!)** — the W++ VS Code extension is **officially back on the Marketplace**!

Despite being mysteriously flagged and removed after reaching **33,000+ downloads**, the extension was eventually re-approved under a new publisher account.

👉 [View W++ on the VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=wlothIndustries.wplusplus)

We’re grateful to the community for the support, memes, and chaos — and we’re more determined than ever to keep W++ alive, weird, and sloth-powered 🦥.

---

## ✨ Features

- ✅ Full tokenizer, parser, and interpreter written in C#
- ✅ Async/await support
- ✅ Lambda expressions (single and multi-param)
- ✅ Control flow: `if`, `else`, `while`, `for`, `switch`
- ✅ Try/catch + `throw` and `return`
- ✅ Custom syntax highlighting and snippets in VSCode
- ✅ The **OOPSIE** model of development (trust us, it’s revolutionary)

---

## 🧠 Why does W++ exist?

This project was created by [Ofek Bickel](https://github.com/sinisterMage) as an educational challenge — to build a real, working language from scratch and share it with the world.

We believe that even chaotic, meme-fueled languages can teach real-world compiler and runtime skills — and spark joy while doing it.


---

## 🧪 Example

wpp
let greet = (name) => {
    print "Hello, " + name;
};

greet("world");

---

## 🆕 NEW!! Homebrew Support 🍺

You can now install the W++ CLI tool (`ingot`) directly on macOS using Homebrew!

### ✅ Quick Install

```bash
brew tap sinistermage/wpp
brew install wpp
```

This installs the `ingot` CLI globally — ready to run `.wpp` scripts from anywhere.

### 🔄 Updates

To upgrade in the future:

```bash
brew upgrade wpp
```

> Powered by ✨ sloths, GitHub Actions, and definitely *not* Visual Basic.


---


## 🐳 New! Official Docker Image

W++ now has an official Docker image!
Run `.wpp` files instantly without installing anything:

```bash
docker run --rm -v "$(pwd)":/wpp sinistermage/wpp myscript.wpp
```

It auto-fetches the latest `ingot` release, works great in CI, and is sloth-approved.
👉 [View on Docker Hub](https://hub.docker.com/r/sinistermage/wpp)

---

## 🦥 New!! The *Actually Functional* W++ Website

So… funny story.
The old W++ website was cute, but let’s be honest — it was held together with duct tape, coffee, and regret.
So I did what any chaotic developer would do: **I built a new one.**

This shiny new site runs on **React**, **Radix UI**, and enough Tailwind to make a CSS purist cry.
It’s faster, prettier, and actually mobile-friendly! (shocking, I know)

### 🌈 What’s New

* 📚 **W++ School** – learn the art of chaotic programming
* ⚙️ **Interactive Docs** – now with scrollbars that actually behave
* 🧠 **Live Playground** – break W++ in real time
* 💬 **Community Hub** – yes, we still have sloth memes
* 💻 **100% React** – no Astro glue this time

🚀 **Check it out here:** [https://wplusplus.org](https://wplusplus.org)

Built for chaos.
Powered by sloths.
**Definitely not Visual Basic.**

(disclaimer: the new site's source code is still not yet avalible, but i will release it soon)

---

## 📁 Project Structure

WPlusPlus/ — Core C# interpreter and AST

IngotCLI/ — CLI wrapper for testing/running .wpp scripts

wpp-vscode/ — VSCode extension with:

Syntax highlighting

Snippets

Icon & metadata

---

## 🤔 Is W++ a Python dialect?

Nope. W++ borrows Python’s readability style, but it is **not** Python or a Python runtime.

- It’s not compatible with Python libraries
- It has a custom syntax, runtime, and execution model
- It compiles to IL and integrates tightly with the .NET ecosystem
- It uses semicolons and braces by design
- It supports NuGet imports — not pip

Think of it as:  
**“.NET scripting with a Python-inspired flavor”** — not “Python on .NET” (that’s IronPython).

📘 [Click here to view the full W++ Syntax Guide](https://github.com/sinisterMage/WPlusPlus/blob/master/SYNTAX.md)


---

## 📜 License

This project is licensed under the MIT License.
Sloth-powered and chaos-approved.








