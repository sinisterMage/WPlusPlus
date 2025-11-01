const vscode = require('vscode');
const path = require('path');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let wppTerminal;
let client;

function getOrCreateTerminal() {
  if (!wppTerminal || wppTerminal.exitStatus !== undefined) {
    wppTerminal = vscode.window.createTerminal("W++");
  }
  return wppTerminal;
}

function activate(context) {
  console.log('[W++ Extension] Activating...');

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

    // Check if we're in an ingot project (has wpp.config.hs)
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
    const fs = require('fs');

    let isIngotProject = false;
    if (workspaceFolder) {
      const wppConfig = path.join(workspaceFolder.uri.fsPath, 'wpp.config.hs');
      isIngotProject = fs.existsSync(wppConfig);
    }

    // If in ingot project, just run "ingot run"
    // Otherwise, run the specific file with "ingot run <filename>"
    if (isIngotProject) {
      terminal.sendText("ingot run");
    } else {
      const fileName = path.basename(filePath);
      terminal.sendText(`ingot run "${fileName}"`);
    }

    terminal.show();
  });

  context.subscriptions.push(runCommand);

  // 🔌 Start Language Server
  const serverModule = context.asAbsolutePath(
    path.join('..', 'wpp-lsp-server', 'server.js')
  );

  const serverOptions = {
    run: { module: serverModule, transport: TransportKind.ipc },
    debug: {
      module: serverModule,
      transport: TransportKind.ipc,
      options: { execArgv: ['--nolazy', '--inspect=6009'] }
    }
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'wpp' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.wpp')
    }
  };

  client = new LanguageClient(
    'wppLanguageServer',
    'W++ Language Server',
    serverOptions,
    clientOptions
  );

  console.log('[W++ Extension] Starting Language Server...');
  client.start();

  context.subscriptions.push(client);
  console.log('[W++ Extension] Activated successfully with LSP');
}

function deactivate() {
  if (wppTerminal) {
    wppTerminal.dispose();
  }

  if (client) {
    return client.stop();
  }
}

module.exports = {
  activate,
  deactivate
};
