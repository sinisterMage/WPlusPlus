-- wpp.config.hs
-- W++ Functional Configuration — Because JSON is for mortals

main :: IO ()
main = do
  entrypoint "src/main.wpp"
  package   "demo"
  version   "1.0.0"
  license   "MIT"
  author    "Ofek Bickel"
  category  "utilities"
  tags      ["cli", "demo"]
  readme    "README.md"
  isPublic  true
  println   "✨ Testing W++ libraries"
