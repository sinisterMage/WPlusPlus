const vscode = require('vscode');
const path = require('path');

let wppTerminal;

function getOrCreateTerminal() {
  if (!wppTerminal || wppTerminal.exitStatus !== undefined) {
    wppTerminal = vscode.window.createTerminal("W++");
  }
  return wppTerminal;
}

function activate(context) {
  // ▶️ Run command
  const runCommand = vscode.commands.registerCommand('wpp.run', async () => {
    const editor = vscode.window.activeTextEditor;

    if (!editor) {
      vscode.window.showErrorMessage("❌ No active editor found.");
      return;
    }

    const filePath = editor.document.fileName;
    if (!filePath.endsWith(".wpp")) {
      vscode.window.showErrorMessage("❌ Please open a `.wpp` file to run W++.");
      return;
    }

    await editor.document.save();
    vscode.window.showInformationMessage("🚀 Running W++ code...");

    const terminal = getOrCreateTerminal();
    terminal.sendText("ingot run");
    terminal.show();
  });

  context.subscriptions.push(runCommand);

  // 💡 IntelliSense completion
  const completionProvider = vscode.languages.registerCompletionItemProvider(
  { scheme: 'file', language: 'wpp' },
  {
    provideCompletionItems(document, position) {
      const linePrefix = document.lineAt(position).text.substr(0, position.character);

      const items = [];

      // Context: inside a print statement
      if (linePrefix.includes("print(")) {
        items.push(new vscode.CompletionItem("\"Hello from W++!\"", vscode.CompletionItemKind.Text));
        items.push(new vscode.CompletionItem("\"DEBUG: value is \" + myVar", vscode.CompletionItemKind.Text));
      }

      // General keywords
      const keywords = [
        "print", "let", "const", "async", "await", "return",
        "if", "else", "while", "for", "break", "continue",
        "true", "false", "null", "import",
        "entity", "inherits", "disown", "birth", "vanish",
        "me", "ancestor", "new", "alters", "switch", "case", "default", "externcall"
      ];

      keywords.forEach(kw => {
        const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
        item.detail = "W++ keyword";
        items.push(item);
      });

      return items;
    }
  },
  '' // Trigger on every character
);


  context.subscriptions.push(completionProvider);
}

function deactivate() {
  if (wppTerminal) {
    wppTerminal.dispose();
  }
}

module.exports = {
  activate,
  deactivate
};
