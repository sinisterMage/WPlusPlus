const {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  InitializeParams,
  TextDocumentSyncKind,
  InitializeResult
} = require('vscode-languageserver/node');

const { TextDocument } = require('vscode-languageserver-textdocument');
const { provideCompletions } = require('./completion');
const { validateDocument } = require('./diagnostics');

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

// Create text document manager
const documents = new TextDocuments(TextDocument);

// Debounce timer for validation
const validationTimers = new Map();

connection.onInitialize((params) => {
  const result = {
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
      completionProvider: {
        resolveProvider: false,
        triggerCharacters: ['.', ':', '{', '(']
      }
    }
  };

  connection.console.log('[W++ LSP] Server initialized');
  return result;
});

connection.onInitialized(() => {
  connection.console.log('[W++ LSP] Server ready');
});

// Text document opened
documents.onDidOpen(e => {
  connection.console.log(`[W++ LSP] Document opened: ${e.document.uri}`);
  validateDocumentDebounced(e.document);
});

// Text document changed
documents.onDidChangeContent(change => {
  connection.console.log(`[W++ LSP] Document changed: ${change.document.uri}`);
  validateDocumentDebounced(change.document);
});

// Text document closed
documents.onDidClose(e => {
  connection.console.log(`[W++ LSP] Document closed: ${e.document.uri}`);
  // Clear diagnostics
  connection.sendDiagnostics({ uri: e.document.uri, diagnostics: [] });
  // Clear validation timer
  const timer = validationTimers.get(e.document.uri);
  if (timer) {
    clearTimeout(timer);
    validationTimers.delete(e.document.uri);
  }
});

/**
 * Debounced document validation
 * Waits 500ms after last change before validating
 */
function validateDocumentDebounced(document) {
  const uri = document.uri;

  // Clear existing timer
  const existingTimer = validationTimers.get(uri);
  if (existingTimer) {
    clearTimeout(existingTimer);
  }

  // Set new timer
  const timer = setTimeout(async () => {
    connection.console.log(`[W++ LSP] Validating: ${uri}`);

    try {
      const diagnostics = await validateDocument(uri, connection);
      connection.sendDiagnostics({ uri, diagnostics });
      connection.console.log(`[W++ LSP] Sent ${diagnostics.length} diagnostic(s) for ${uri}`);
    } catch (error) {
      connection.console.error(`[W++ LSP] Validation error: ${error.message}`);
    }

    validationTimers.delete(uri);
  }, 500); // 500ms debounce

  validationTimers.set(uri, timer);
}

// Code completion
connection.onCompletion((textDocumentPosition) => {
  const document = documents.get(textDocumentPosition.textDocument.uri);

  if (!document) {
    return [];
  }

  const position = textDocumentPosition.position;
  connection.console.log(`[W++ LSP] Completion request at ${position.line}:${position.character}`);

  try {
    const completions = provideCompletions(document, position);
    connection.console.log(`[W++ LSP] Returning ${completions.length} completion items`);
    return completions;
  } catch (error) {
    connection.console.error(`[W++ LSP] Completion error: ${error.message}`);
    return [];
  }
});

// Make the text document manager listen on the connection
documents.listen(connection);

// Listen on the connection
connection.listen();

connection.console.log('[W++ LSP] Server started and listening');
