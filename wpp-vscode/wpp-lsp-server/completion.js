const { CompletionItem, CompletionItemKind, InsertTextFormat } = require('vscode-languageserver/node');
const { WPP_KEYWORDS } = require('./utils');

/**
 * Keyword completion items
 */
function getKeywordCompletions() {
  const items = [];

  // Create completion items for all W++ keywords
  WPP_KEYWORDS.forEach(keyword => {
    const item = CompletionItem.create(keyword);
    item.kind = CompletionItemKind.Keyword;
    item.detail = 'W++ keyword';

    // Add documentation for important keywords
    switch (keyword) {
      case 'funcy':
        item.documentation = 'Define a function (W++ style)\n\nExample:\nfuncy add(a: int, b: int) -> int {\n  return a + b\n}';
        item.insertText = 'funcy ${1:name}(${2:params}) {\n  ${3}\n}';
        item.insertTextFormat = InsertTextFormat.Snippet;
        break;
      case 'try':
        item.documentation = 'Begin exception handling block';
        item.insertText = 'try {\n  ${1}\n} catch (${2:e}) {\n  ${3}\n}';
        item.insertTextFormat = InsertTextFormat.Snippet;
        break;
      case 'entity':
        item.documentation = 'Define an entity (OOP class)\n\nExample:\nentity Dog {\n  name: string\n}';
        item.insertText = 'entity ${1:Name} {\n  ${2}\n}';
        item.insertTextFormat = InsertTextFormat.Snippet;
        break;
      case 'type':
        item.documentation = 'Define a type alias\n\nExample:\ntype User = {\n  name: string,\n  age: int\n}';
        item.insertText = 'type ${1:Name} = {\n  ${2}\n}';
        item.insertTextFormat = InsertTextFormat.Snippet;
        break;
      case 'import':
        item.documentation = 'Import symbols from another module';
        item.insertText = 'import { ${1:symbols} } from "${2:module}"';
        item.insertTextFormat = InsertTextFormat.Snippet;
        break;
      case 'export':
        item.documentation = 'Export symbol for use in other modules';
        break;
      case 'async':
        item.documentation = 'Mark function as asynchronous';
        break;
      case 'await':
        item.documentation = 'Await an async operation';
        break;
      case 'throw':
        item.documentation = 'Throw an exception';
        item.insertText = 'throw ${1:error}';
        item.insertTextFormat = InsertTextFormat.Snippet;
        break;
      case 'alters':
        item.documentation = 'Inherit from parent entity\n\nExample:\nentity Dog alters Animal { ... }';
        break;
    }

    items.push(item);
  });

  return items;
}

/**
 * Context-aware completions based on current line
 */
function getContextCompletions(lineText, position) {
  const items = [];
  const beforeCursor = lineText.substring(0, position.character);

  // Inside try block - suggest catch/finally
  if (beforeCursor.includes('try') && !beforeCursor.includes('catch')) {
    const catchItem = CompletionItem.create('catch');
    catchItem.kind = CompletionItemKind.Keyword;
    catchItem.detail = 'Catch exception';
    catchItem.insertText = 'catch (${1:e}) {\n  ${2}\n}';
    catchItem.insertTextFormat = InsertTextFormat.Snippet;
    items.push(catchItem);

    const finallyItem = CompletionItem.create('finally');
    finallyItem.kind = CompletionItemKind.Keyword;
    finallyItem.detail = 'Finally block';
    finallyItem.insertText = 'finally {\n  ${1}\n}';
    finallyItem.insertTextFormat = InsertTextFormat.Snippet;
    items.push(finallyItem);
  }

  // After import - suggest from
  if (beforeCursor.trim().endsWith('import {') || beforeCursor.includes('import {') && !beforeCursor.includes('from')) {
    const fromItem = CompletionItem.create('from');
    fromItem.kind = CompletionItemKind.Keyword;
    fromItem.detail = 'Specify module path';
    fromItem.insertText = '} from "${1:module}"';
    fromItem.insertTextFormat = InsertTextFormat.Snippet;
    items.push(fromItem);
  }

  // After entity name - suggest alters
  if (/entity\s+\w+/.test(beforeCursor) && !beforeCursor.includes('{') && !beforeCursor.includes('alters')) {
    const altersItem = CompletionItem.create('alters');
    altersItem.kind = CompletionItemKind.Keyword;
    altersItem.detail = 'Inherit from parent entity';
    altersItem.insertText = 'alters ${1:Parent}';
    altersItem.insertTextFormat = InsertTextFormat.Snippet;
    items.push(altersItem);
  }

  return items;
}

/**
 * Provide completion items for the given position
 */
function provideCompletions(document, position) {
  const text = document.getText();
  const lines = text.split('\n');
  const currentLine = lines[position.line] || '';

  const keywordItems = getKeywordCompletions();
  const contextItems = getContextCompletions(currentLine, position);

  return [...keywordItems, ...contextItems];
}

module.exports = {
  provideCompletions
};
