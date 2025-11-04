#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
CLI_BIN="$ROOT_DIR/target/release/wpp-cli"

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "ERROR: hyperfine not found on PATH. Install hyperfine to run this script." >&2
  exit 1
fi

echo "== Building W++ CLI (release) =="
cargo build -q -p wpp-cli --release

echo
echo "== Hyperfine: LCG (50M steps, Int32) =="
# Prebuild C# projects once
(cd "$ROOT_DIR/benchmarks/csharp-lcg" && dotnet build -c Release >/dev/null)

hyperfine -i -w 1 -r 5 \
  "cd $ROOT_DIR/benchmarks/wpp-lcg && $CLI_BIN run --opt" \
  "cd $ROOT_DIR && julia benchmarks/julia/lcg.jl" \
  "cd $ROOT_DIR && N=50000000 python3 benchmarks/python/lcg.py" \
  "cd $ROOT_DIR && N=50000000 ruby benchmarks/ruby/lcg.rb" \
  "cd $ROOT_DIR/benchmarks/csharp-lcg && dotnet run -c Release --no-build" \
  "cd $ROOT_DIR && N=50000000 lua benchmarks/lua/lcg.lua" \
  "cd $ROOT_DIR && N=50000000 bun benchmarks/js/lcg.js"

echo
echo "== Hyperfine: Branch (50M iters) =="
(cd "$ROOT_DIR/benchmarks/csharp-branch" && dotnet build -c Release >/dev/null)

hyperfine -i -w 1 -r 5 \
  "cd $ROOT_DIR/benchmarks/wpp-branch && $CLI_BIN run --opt" \
  "cd $ROOT_DIR && julia benchmarks/julia/branch.jl" \
  "cd $ROOT_DIR && N=50000000 python3 benchmarks/python/branch.py" \
  "cd $ROOT_DIR && N=50000000 ruby benchmarks/ruby/branch.rb" \
  "cd $ROOT_DIR/benchmarks/csharp-branch && dotnet run -c Release --no-build" \
  "cd $ROOT_DIR && N=50000000 lua benchmarks/lua/branch.lua" \
  "cd $ROOT_DIR && N=50000000 bun benchmarks/js/branch.js"

echo
echo "== Hyperfine: Fibonacci(45) =="
(cd "$ROOT_DIR/benchmarks/csharp-fib" && dotnet build -c Release >/dev/null)

hyperfine -i -w 1 -r 10 \
  "cd $ROOT_DIR/benchmarks/wpp-fib && $CLI_BIN run --opt" \
  "cd $ROOT_DIR && julia benchmarks/julia/fib.jl" \
  "cd $ROOT_DIR && python3 benchmarks/python/fib.py" \
  "cd $ROOT_DIR && ruby benchmarks/ruby/fib.rb" \
  "cd $ROOT_DIR/benchmarks/csharp-fib && dotnet run -c Release --no-build" \
  "cd $ROOT_DIR && lua benchmarks/lua/fib.lua" \
  "cd $ROOT_DIR && bun benchmarks/js/fib.js"

echo
echo "Done."
