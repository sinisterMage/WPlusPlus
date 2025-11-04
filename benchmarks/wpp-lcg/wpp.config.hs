-- wpp.config.hs for W++ LCG benchmark

main :: IO ()
main = do
  entrypoint "src/main.wpp"
  package   "wpp-bench-lcg"
  version   "1.0.0"
  license   "MIT"
  author    "Bench"
  println  "LCG benchmark project loaded."

