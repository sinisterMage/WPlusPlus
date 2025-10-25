# 🦥 W++ Official Docker Image

Run **W++**, the chaotic yet powerful scripting language, directly from Docker — no installs, no setup.

---

## 🧱 What’s inside

This image automatically downloads the **latest W++ CLI (`wpp`)** from [GitHub Releases](https://github.com/sinisterMage/WPlusPlus/releases) and installs it inside a minimal **Ubuntu 24.04** base image (glibc 2.39).
It comes ready to execute `.wpp` scripts instantly, with full support for async execution and GC-managed multithreading.

---

## 🚀 Usage

### Run a local `.wpp` file

```bash
docker run --rm -v $(pwd):/app sinistermage/wplusplus my_script.wpp
```

### Check version

```bash
docker run --rm sinistermage/wplusplus --version
```

---

## 🧩 Volumes & working directory

By default:

* Working directory: `/app`
* Mount local code with `-v $(pwd):/app` to make it accessible inside the container.

---

## 🧠 Example: Hello World

```bash
echo 'print("Hello from W++ 🦥")' > hello.wpp
docker run --rm -v $(pwd):/app sinistermage/wplusplus hello.wpp
```

Output:

```
Hello from W++ 🦥
```

---

## ⚙️ Architecture Support

* ✅ `x86_64` (Intel/AMD)
* 🧩 ARM64 (Apple Silicon) support planned

---

## 🧰 Base Image

* **Ubuntu 24.04 (glibc 2.39)**
* Includes: `curl`, `jq`, `tar`, `ca-certificates`

---

## 💡 Environment Variables

| Variable        | Description                                        | Default |
| --------------- | -------------------------------------------------- | ------- |
| `WPP_DEBUG`     | Enables verbose logging (compilation & GC traces)  | `0`     |
| `WPP_SAFE_MODE` | Restricts runtime features for sandboxed execution | `0`     |

Example:

```bash
docker run --rm -e WPP_DEBUG=1 -v $(pwd):/app sinistermage/wplusplus my_script.wpp
```

---

## ⚡ W++ Highlights

* 🦥 **GC-managed multithreading** — concurrency that *mostly works*, powered by a custom garbage collector.
* 🧠 **Multiple dispatch** — smarter than overloading.
* 🧩 **Async / Await** — asynchronous chaos, W++-style.
* 💬 **UTF-8 identifiers** — variable names can literally be emoji.
* 🕹️ **OOPSIE Framework** — Object-Oriented Pseudo-Structural Inheritance Engine™.
* 🌀 **Cross-target support** — runs as interpreter, JIT, or LLVM-compiled binary.

---

## 📦 Build locally

```bash
git clone https://github.com/sinisterMage/WPlusPlus.git
cd WPlusPlus/Docker
docker build -t wplusplus:latest .
```

---

## 🧡 About W++

W++ is a chaotic, Python-style scripting language for .NET and LLVM — blending expressive syntax, GC-based multithreading, and serious compiler tech with a sloth’s attitude toward productivity.
👉 [Learn more on GitHub](https://github.com/sinisterMage/WPlusPlus)
