
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

## 🔮 Roadmap

* [x] LLVM-based backend
* [x] Core language ported
* [x] Ingot CLI integration
* [x] FreeBSD builds (because why not)
* [ ] HTTP & API support
* [ ] Async / Await
* [ ] OOPSIE Framework ™
* [ ] Windows support (eventually…)

---

### 🦥 Philosophy

W++ was never meant to be perfect.
It’s meant to be *fun*, *chaotic*, and *educational* — a language that teaches compilers by constantly trying to destroy them.
If you can build something in W++ v2 and it doesn’t segfault, that’s a feature.

---

**Go forth, compile chaos, and make the sloth proud.**

— Ofek “sinisterMage” Bickel 🦥
*Creator of W++ | Professional Chaos Engineer*











