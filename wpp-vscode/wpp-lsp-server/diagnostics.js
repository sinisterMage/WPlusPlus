const { exec } = require('child_process');
const { Diagnostic, DiagnosticSeverity } = require('vscode-languageserver/node');
const { getFileDirectory, getFilePath, isIngotAvailable } = require('./utils');
const path = require('path');

/**
 * Parse error messages from ingot output
 */
function parseIngotErrors(stderr, filePath) {
  const diagnostics = [];
  const lines = stderr.split('\n');

  for (const line of lines) {
    // Try to parse error format: "file.wpp:line:col: error: message"
    // or simpler: "error: message"
    const match = line.match(/(?:(\d+):(\d+):)?\s*(error|warning|info):\s*(.+)/i);

    if (match) {
      const [, lineNum, colNum, severity, message] = match;

      const diagnostic = {
        severity: severity.toLowerCase() === 'error' ? DiagnosticSeverity.Error :
                  severity.toLowerCase() === 'warning' ? DiagnosticSeverity.Warning :
                  DiagnosticSeverity.Information,
        range: {
          start: {
            line: lineNum ? parseInt(lineNum) - 1 : 0,
            character: colNum ? parseInt(colNum) - 1 : 0
          },
          end: {
            line: lineNum ? parseInt(lineNum) - 1 : 0,
            character: colNum ? parseInt(colNum) + 10 : 100
          }
        },
        message: message.trim(),
        source: 'W++ (ingot)'
      };

      diagnostics.push(diagnostic);
    }
  }

  // If no structured errors found but stderr has content, create generic error
  if (diagnostics.length === 0 && stderr.trim()) {
    // Look for common error patterns
    if (stderr.includes('parse') || stderr.includes('Parse') || stderr.includes('unexpected')) {
      diagnostics.push({
        severity: DiagnosticSeverity.Error,
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 100 }
        },
        message: `Parse error: ${stderr.split('\n')[0].trim()}`,
        source: 'W++ (ingot)'
      });
    } else if (stderr.includes('Error') || stderr.includes('error')) {
      diagnostics.push({
        severity: DiagnosticSeverity.Error,
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 100 }
        },
        message: stderr.split('\n')[0].trim(),
        source: 'W++ (ingot)'
      });
    }
  }

  return diagnostics;
}

/**
 * Check if directory has ingot project config (wpp.config.hs)
 */
function isIngotProject(dir) {
  const fs = require('fs');
  const wppConfig = path.join(dir, 'wpp.config.hs');
  return fs.existsSync(wppConfig);
}

/**
 * Validate W++ document using ingot
 */
function validateDocument(documentUri, connection) {
  return new Promise((resolve) => {
    if (!isIngotAvailable()) {
      // Ingot not available - no diagnostics (silent failure)
      resolve([]);
      return;
    }

    const filePath = getFilePath(documentUri);
    const fileDir = getFileDirectory(documentUri);
    const fileName = path.basename(filePath);

    // Check if we're in an ingot project
    const inProject = isIngotProject(fileDir);

    let command;
    if (inProject) {
      // In a project - try "ingot check" first (if available)
      // This validates without running
      command = 'ingot check';
    } else {
      // Standalone file - run with filename
      command = `ingot run "${fileName}"`;
    }

    // Run ingot to check for errors
    exec(command, {
      cwd: fileDir,
      timeout: 5000 // 5 second timeout
    }, (error, stdout, stderr) => {
      let diagnostics = [];

      if (error) {
        // Check if "ingot check" is not available (command not found)
        if (inProject && (stderr.includes('Unknown command') || stderr.includes('not found') ||
            stderr.includes('check is not') || error.code === 127)) {
          // "ingot check" not available, try with "ingot run" instead
          exec('ingot run', {
            cwd: fileDir,
            timeout: 5000
          }, (runError, runStdout, runStderr) => {
            let runDiagnostics = [];
            if (runError) {
              // Only show parse/syntax errors, not runtime errors
              if (runStderr.includes('parse') || runStderr.includes('Parse') ||
                  runStderr.includes('syntax') || runStderr.includes('Syntax') ||
                  runStderr.includes('unexpected token') || runStderr.includes('expected') ||
                  runStderr.includes('error:')) {
                runDiagnostics = parseIngotErrors(runStderr, filePath);
              }
            }
            resolve(runDiagnostics);
          });
          return;
        }

        // Parse errors from stderr (for both check and run commands)
        // Only show parse/syntax/type errors, not runtime errors
        if (stderr.includes('parse') || stderr.includes('Parse') ||
            stderr.includes('syntax') || stderr.includes('Syntax') ||
            stderr.includes('unexpected token') || stderr.includes('expected') ||
            stderr.includes('type error') || stderr.includes('Type error') ||
            stderr.includes('error:')) {

          // Filter out runtime errors like "Module not found"
          if (!stderr.includes('Module') && !stderr.includes('panicked') &&
              !stderr.includes('Failed to load')) {
            diagnostics = parseIngotErrors(stderr, filePath);
          }
        }
      }

      resolve(diagnostics);
    });
  });
}

module.exports = {
  validateDocument,
  parseIngotErrors
};
