const { execSync } = require('child_process');
const path = require('path');

/**
 * W++ keywords for completion (synced with lexer.rs)
 */
const WPP_KEYWORDS = [
  'let', 'const', 'if', 'else', 'while', 'for', 'break', 'continue',
  'true', 'false', 'switch', 'case', 'default', 'try', 'catch', 'throw', 'finally',
  'funcy', 'func', 'return', 'async', 'await',
  'entity', 'alters', 'me', 'new',
  'import', 'export', 'from', 'type'
];

/**
 * Check if ingot CLI is available
 */
function isIngotAvailable() {
  try {
    execSync('which ingot', { stdio: 'ignore' });
    return true;
  } catch (error) {
    return false;
  }
}

/**
 * Get file directory for running ingot commands
 */
function getFileDirectory(uri) {
  const filePath = uri.replace('file://', '');
  return path.dirname(filePath);
}

/**
 * Get file path from URI
 */
function getFilePath(uri) {
  return uri.replace('file://', '');
}

module.exports = {
  WPP_KEYWORDS,
  isIngotAvailable,
  getFileDirectory,
  getFilePath
};
