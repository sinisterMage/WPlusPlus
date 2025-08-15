# 🐳 W++ Docker Image

The official Docker image for [W++](https://github.com/sinisterMage/WPlusPlus), the Python-style scripting language for .NET — powered by chaos, memes, and sloths 🦥.

## 📦 What’s Inside?

This image includes the latest release of the `ingot` CLI tool, configured to run `.wpp` scripts directly using:

```

ingot run-file <your-file>.wpp

````

## 🚀 How to Use

Run a `.wpp` script in the current directory:

```bash
docker run --rm -v "$(pwd)":/wpp sinistermage/wpp myscript.wpp
````

> 📝 You can also alias it:
>
> ```bash
> alias wpp="docker run --rm -v \"\$PWD\":/wpp sinistermage/wpp"
> wpp myscript.wpp
> ```

### 🛠️ Building Manually (for devs)

```bash
docker build --platform=linux/amd64 -t wpp ./Docker
```

### 📤 Running with ARM or emulation

```bash
docker run --platform=linux/amd64 --rm -v "$(pwd)":/wpp sinistermage/wpp myscript.wpp
```

## 🤯 Features

* 🧠 Full support for HTTP requests
* 🗃️ Built-in GC
* ⚔️ Multiple dispatch
* 🔥 Interpreter + JIT
* 🐧 Works on Linux/macOS via Docker (no .NET install needed!)

## 📄 License

W++ is MIT licensed. Go wild.

---

W++: the language you never knew you needed until it printed “hello world” while nuking sanity.

