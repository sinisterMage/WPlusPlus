-- wpp.config.hs for W++ branch-heavy benchmark

main :: IO ()
main = do
  entrypoint "src/main.wpp"
  package   "wpp-bench-branch"
  version   "1.0.0"
  license   "MIT"
  author    "Bench"
  println  "Branch benchmark project loaded."

