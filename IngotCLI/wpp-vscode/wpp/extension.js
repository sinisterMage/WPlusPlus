const vscode = require('vscode');

function activate(context) {
  const runCommand = vscode.commands.registerCommand('wpp.run', () => {
    const terminal = vscode.window.createTerminal("W++");
    terminal.sendText("ingot run");
    terminal.show();
  });

  context.subscriptions.push(runCommand);
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
