#!/usr/bin/env bash
set -euo pipefail

# Simple portable runner that measures wall time with POSIX `time -p`.
# It prints the median of 3 runs for each test.

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
CLI_BIN="$ROOT_DIR/target/release/wpp-cli"

if ! command -v julia >/dev/null 2>&1; then
  echo "ERROR: julia not found on PATH. Install Julia to run comparisons." >&2
  exit 1
fi

echo "== Building W++ CLI (release) =="
cargo build -q -p wpp-cli --release

# Resolve a reliable `time` implementation
TIME_BIN="/usr/bin/time"
if [ ! -x "$TIME_BIN" ]; then
  if command -v gtime >/dev/null 2>&1; then
    TIME_BIN="$(command -v gtime)"
  else
    TIME_BIN="time"
  fi
fi

measure() {
  # Args: <dir> <cmd...>
  local dir="$1"; shift
  local -a samples=()
  for _ in 1 2 3; do
    # Run command with stdout suppressed, capture stderr (time -p prints to stderr)
    local out
    out=$((cd "$dir" && $TIME_BIN -p "$@" 1>/dev/null) 2>&1 | awk '/^real/ {print $2}')
    samples+=("$out")
  done
  # sort and take median (2nd of 3)
  printf '%s\n' "${samples[@]}" | sort -n | sed -n '2p'
}

printf "\n== Benchmark: LCG (50M steps, Int32) ==\n"
WPP_LCG=$(measure "$ROOT_DIR/benchmarks/wpp-lcg" "$CLI_BIN" run --opt)
JL_LCG=$(measure "$ROOT_DIR" julia benchmarks/julia/lcg.jl || echo "err")
PY_LCG=$(measure "$ROOT_DIR" env N=5000000 python3 benchmarks/python/lcg.py)
RB_LCG=$(measure "$ROOT_DIR" env N=5000000 ruby benchmarks/ruby/lcg.rb)
(cd "$ROOT_DIR/benchmarks/csharp-lcg" && dotnet build -c Release >/dev/null)
CS_LCG=$(measure "$ROOT_DIR/benchmarks/csharp-lcg" dotnet run -c Release --no-build)
LU_LCG=$(measure "$ROOT_DIR" env N=5000000 lua benchmarks/lua/lcg.lua || echo "err")
BN_LCG=$(measure "$ROOT_DIR" env N=5000000 bun benchmarks/js/lcg.js || echo "err")
printf "W++   (s): %s\n" "$WPP_LCG"
printf "Julia (s): %s\n" "$JL_LCG"
printf "Py    (s): %s (N=5M)\n" "$PY_LCG"
printf "Ruby  (s): %s (N=5M)\n" "$RB_LCG"
printf "C#    (s): %s\n" "$CS_LCG"
printf "Lua   (s): %s (N=5M)\n" "$LU_LCG"
printf "Bun   (s): %s (N=5M)\n" "$BN_LCG"

printf "\n== Benchmark: Branch (50M iters) ==\n"
WPP_BR=$(measure "$ROOT_DIR/benchmarks/wpp-branch" "$CLI_BIN" run --opt)
JL_BR=$(measure "$ROOT_DIR" julia benchmarks/julia/branch.jl || echo "err")
PY_BR=$(measure "$ROOT_DIR" env N=50000000 python3 benchmarks/python/branch.py)
RB_BR=$(measure "$ROOT_DIR" env N=50000000 ruby benchmarks/ruby/branch.rb)
(cd "$ROOT_DIR/benchmarks/csharp-branch" && dotnet build -c Release >/dev/null)
CS_BR=$(measure "$ROOT_DIR/benchmarks/csharp-branch" dotnet run -c Release --no-build)
LU_BR=$(measure "$ROOT_DIR" env N=50000000 lua benchmarks/lua/branch.lua || echo "err")
BN_BR=$(measure "$ROOT_DIR" env N=50000000 bun benchmarks/js/branch.js || echo "err")
printf "W++   (s): %s\n" "$WPP_BR"
printf "Julia (s): %s\n" "$JL_BR"
printf "Py    (s): %s (N=5M)\n" "$PY_BR"
printf "Ruby  (s): %s (N=5M)\n" "$RB_BR"
printf "C#    (s): %s\n" "$CS_BR"
printf "Lua   (s): %s (N=5M)\n" "$LU_BR"
printf "Bun   (s): %s (N=5M)\n" "$BN_BR"

printf "\n== Benchmark: Fibonacci(45) ==\n"
WPP_FB=$(measure "$ROOT_DIR/benchmarks/wpp-fib" "$CLI_BIN" run --opt)
JL_FB=$(measure "$ROOT_DIR" julia benchmarks/julia/fib.jl || echo "err")
PY_FB=$(measure "$ROOT_DIR" python3 benchmarks/python/fib.py)
RB_FB=$(measure "$ROOT_DIR" ruby benchmarks/ruby/fib.rb)
(cd "$ROOT_DIR/benchmarks/csharp-fib" && dotnet build -c Release >/dev/null)
CS_FB=$(measure "$ROOT_DIR/benchmarks/csharp-fib" dotnet run -c Release --no-build)
LU_FB=$(measure "$ROOT_DIR" lua benchmarks/lua/fib.lua || echo "err")
BN_FB=$(measure "$ROOT_DIR" bun benchmarks/js/fib.js || echo "err")
printf "W++   (s): %s\n" "$WPP_FB"
printf "Julia (s): %s\n" "$JL_FB"
printf "Py    (s): %s\n" "$PY_FB"
printf "Ruby  (s): %s\n" "$RB_FB"
printf "C#    (s): %s\n" "$CS_FB"
printf "Lua   (s): %s\n" "$LU_FB"
printf "Bun   (s): %s\n" "$BN_FB"

echo "\nDone."
