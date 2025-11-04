-- wpp.config.hs for W++ Fibonacci(45) benchmark

main :: IO ()
main = do
  entrypoint "src/main.wpp"
  package   "wpp-bench-fib"
  version   "1.0.0"
  license   "MIT"
  author    "Bench"
  println  "Fibonacci benchmark project loaded."

