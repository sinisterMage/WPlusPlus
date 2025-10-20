
# W++ 🦥

![image](https://github.com/user-attachments/assets/e55dc88e-7ef0-4aa6-8d3e-fbb77c9aac08)
![W++ LLVM](https://img.shields.io/badge/W%2B%2B%20v2-LLVM%20Powered-orange?style=flat-square\&logo=rust\&logoColor=white)
![Extension: Resurrected](https://img.shields.io/badge/W%2B%2B%20Extension-Resurrected-purple?style=flat-square\&logo=github\&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

> *Built with chaos. Forged by sloths. Rewritten in Rust.*

---

## 🧠 Welcome to W++ v2 — The LLVM Era

W++ v2 marks the full rebirth of the **sloth-powered scripting language** you never asked for.
The old C# interpreter has retired peacefully, and in its place rises a **real compiler** — built with **Rust**, targeting **LLVM**, and powered by *questionable life choices*.

This isn’t just a rewrite. It’s a declaration that W++ is officially moving from “toy” to “terrifyingly functional.”

---

## ⚙️ What Makes v2 Different?

| Old W++                            | New W++ (v2)                              |
| ---------------------------------- | ----------------------------------------- |
| ☠️ C# interpreter with async tears | 🦀 Rust + LLVM-backed compiler            |
| Heavy .NET runtime                 | Native machine code, zero dependencies    |
| JIT-ish execution                  | True LLVM IR + optional JIT               |
| Managed chaos                      | Unmanaged chaos                           |
| *Maybe* portable                   | Actually portable (Linux, macOS, FreeBSD) |

---

## 🧩 Core Features

* `let` declarations & expressions
* `print` (via native `printf`)
* `if / else`, `while`, `for`, `break`, `continue`
* User-defined functions
* Basic exception globals (for your inevitable mistakes)

All compiled directly into **LLVM IR** and optimized by the same backend that powers Clang and Rust.
(Yes, your memes now run at native speed.)

---

## 💾 Installation

1. Download the latest binary for your platform from [**Releases**](https://github.com/sinisterMage/WPlusPlus/releases).
2. Extract it somewhere convenient.
3. Add it to your system `PATH`.
4. Run a file:

   ```bash
   ingot run hello.wpp
   ```

If it explodes, congratulations — you’re using it correctly.

---

## 🪟 About Windows Support

At the moment, LLVM and Windows are not on speaking terms.
Native binaries will return once peace negotiations succeed.

Until then:

* Use **WSL** 🐧
* Or a **Linux VM**
* Or simply accept your fate

---

## 💻 Installing W++ on a Chromebook (aka, how to void your warranty)

So… you’re on a Chromebook and thought:

> “Yeah, I totally need a sloth-powered LLVM compiler on my browser laptop.”

Respect. Here’s how to make it happen:

1. **Enable Linux (Crostini)**
   Open Settings → Advanced → Developers → **Turn On Linux (Beta)**
   (If it’s greyed out — sorry, your school’s IT admin already hates fun.)

2. **Open the Terminal**
   That scary black window that says “Penguin 🐧” — that’s the one.
   Don’t panic if it asks for updates. Panic if it doesn’t.

3. **Clone the repo manually**
   Since there’s no fancy install script (yet 😭), you’ll have to go old-school:

   ```bash
   sudo apt update
   sudo apt install -y git build-essential llvm-15 clang-15
   git clone https://github.com/sinisterMage/WPlusPlus.git
   cd WPlusPlus
   cargo build --release -p wpp-cli
   ```

4. **Add W++ to PATH (optional but makes you look professional)**

   ```bash
   echo 'export PATH="$PATH:$HOME/WPlusPlus/target/release"' >> ~/.bashrc
   source ~/.bashrc
   ```

5. **Run your first chaotic program**

   ```bash
   ingot run hello.wpp
   ```

   If it prints something — congrats!
   You’ve just compiled a programming language on a Chromebook.
   (Your fans are now operating at NASA levels.)

> ⚠️ Disclaimer: W++ may cause your Chromebook to question its existence.
> Please keep snacks nearby for emotional support.

---

## 🧬 Tech Stack

* **Language core:** Rust 🦀
* **Backend:** LLVM 15 via `inkwell` + `llvm-sys`
* **CLI:** `ingot`, now a standalone binary calling the compiler as a library
* **Optimization:** Optional passes for JIT & builds
* **Design goal:** “It compiles and it’s funny.”

---

## ❤️ Credits

* **LLVM Project** – for existing and making my life difficult (still love you 💕)
* **Rust community** – for turning panic messages into poetry
* **Wloth the Sloth** – for approving every commit at 0.2× speed

---

## 🗺️ W++ Roadmap — *The Path to Controlled Chaos*

---

### 🦥 **Release 1.0 — The LLVM Awakens (First Stable Release)**

* The first-ever **stable release of W++ v2** is finally here!
  Built entirely in **Rust**, powered by **LLVM**, and running at *native chaotic speed*.
* Full rewrite from the old C# interpreter — now with:

  * UTF-8 variable names
  * Lambda support
  * Multiple dispatch
  * The OOPSIE Framework™
  * The Ingot Package Registry (`ingotwpp.dev`)
* If it runs without segfaulting, that’s a feature.
* If it doesn’t — that’s tradition.

> “It compiles! Probably!”

---

### 🧩 **Planned Libraries (a.k.a. The Slothverse Expansion Pack)**

| Library                  | Description                                                            |
| ------------------------ | ---------------------------------------------------------------------- |
| 🧠 **JSON**              | Native JSON parsing and serialization                                  |
| 🕸️ **CORS**             | Cross-Origin (and Cross-Dimensional) Request Support                   |
| 🗄️ **DB Drivers**       | MySQL, PostgreSQL, MongoDB, Firebase, and Apache Cassandra             |
| ☀️ **Proxima Notebooks** | Jupyter-style notebooks set in a solar system — each planet = notebook |
| 🕰️ **Pascal Interop**   | For those who miss `begin` and `end`                                   |
| 🔐 **Wpp.bycrypt**       | bcrypt for chaotic authentication                                      |
| ⚡ **Raython**            | Full-stack API framework powered by the W++ GC thread model            |
| 💾 **wpp.IO**            | File system & I/O utilities                                            |
| 🧠 **wpp.WebGPU**        | Direct interaction with the future (no OpenGL allowed)                 |
| 🌀 **is-odd**            | Implied by the name, but it *will* work                                |
| ☁️ **wpp.IaC**           | Infrastructure as Chaos — deploy W++ code to the cloud                 |
| 💬 **discord.wpp**       | Discord integration for the W++ runtime                                |

> 🧩 **First-Party Library Guidelines**
>
> 1. All first-party W++ libraries will be **open-sourced** under an **OSI-approved permissive license**.
> 2. All first-party libraries will **actually function**.
>    Yes — even `is-odd`.

---

### 🧃 Future Dreams / Threats

* `async drop` for emotionally detached memory management
* Native graphics API (`draw.rect`, `draw.wloth`)
* W++ Cloud ☁️ — serverless, stateless, sanity-less
* Quantum support (once we figure out what “entangled null” means)


> 🦥 *“W++ will never be finished — only temporarily stable.”*

---


### 🦥 Philosophy

W++ was never meant to be perfect.
It’s meant to be *fun*, *chaotic*, and *educational* — a language that teaches compilers by constantly trying to destroy them.
If you can build something in W++ v2 and it doesn’t segfault, that’s a feature.

---

**Go forth, compile chaos, and make the sloth proud.**

— Ofek “sinisterMage” Bickel 🦥
*Creator of W++ | Professional Chaos Engineer*











