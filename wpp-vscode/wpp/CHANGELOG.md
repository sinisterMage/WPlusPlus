# Change Log

All notable changes to the W++ VSCode extension.

## [0.1.0] - 2025-11-01

### Added
- **Language Server Protocol (LSP) Support**
  - Real-time error diagnostics via `ingot` compiler
  - Smart code completion with context awareness
  - Snippet-based completions for common patterns

- **Updated Keywords (W++ v2)**
  - Added: `try`, `catch`, `throw`, `finally` (exception handling)
  - Added: `funcy`, `func` (function declarations)
  - Added: `export`, `from` (module system)
  - Added: `type` (type aliases)
  - Removed obsolete keywords: `inherits`, `disown`, `birth`, `vanish`, `ancestor`, `externcall`

- **Enhanced Code Snippets**
  - Try-catch-finally exception handling
  - Funcy function declarations with type annotations
  - Type alias definitions
  - Updated import/export syntax
  - Entity inheritance with `alters`

### Changed
- Synchronized all keywords with W++ v2 lexer (31 keywords total)
- Improved TextMate grammar for better syntax highlighting
- Updated extension description to reflect LSP features

### Fixed
- Removed `null` as keyword (not in W++ v2 lexer)
- Corrected entity syntax in snippets

## [0.0.7] - Previous

- Basic syntax highlighting
- Simple IntelliSense with hardcoded keywords
- `ingot run` command integration
- File icons and snippets