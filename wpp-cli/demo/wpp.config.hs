-- wpp.config.hs
-- W++ Functional Configuration — Because JSON is for mortals

main :: IO ()
main = do
  entrypoint "src/main.wpp"
  package   "demo"
  version   "1.0.0"
  license   "MIT"
  author    "Ofek Bickel"
  println  "✨ Config loaded successfully. Chaos imminent."
