// public/codemirror-wpp.js
(function (mod) {
  if (typeof exports === "object" && typeof module === "object")
    mod(require("codemirror"));
  else if (typeof define === "function" && define.amd)
    define(["codemirror"], mod);
  else
    mod(window.CodeMirror);
})(function (CodeMirror) {
  // codemirror-wpp.js
// Requires CodeMirror 5 to be loaded

CodeMirror.defineMode("wpp", function () {
  const keywords = new Set([
    "let", "const", "if", "else", "while", "for", "return", "break", "continue",
    "switch", "case", "default", "try", "catch", "throw", "await", "typeof",
    "new", "me", "entity", "alters", "disown"
  ]);

  const builtins = new Set([
    "print", "text", "json", "http"
  ]);

  const atoms = new Set(["null", "true", "false"]);

  const operators = /^(?:==|!=|<=|>=|&&|\|\||\?\?|[+\-*\/<>!=])/;

  return {
    startState: function () {
      return { inString: false, stringType: null };
    },

    token: function (stream, state) {
      // Skip spaces
      if (stream.eatSpace()) return null;

      // Comments
      if (stream.match("//")) {
        stream.skipToEnd();
        return "comment";
      }

      // Strings
      if (!state.inString && (stream.match('"') || stream.match("'"))) {
        state.inString = true;
        state.stringType = stream.current();
        return "string";
      }

      if (state.inString) {
        if (stream.skipTo(state.stringType)) {
          stream.next();
          state.inString = false;
        } else {
          stream.skipToEnd();
        }
        return "string";
      }

      // Numbers
      if (stream.match(/^\d+(\.\d+)?/)) {
        return "number";
      }

      // Operators
      if (stream.match(operators)) {
        return "operator";
      }

      // Identifiers
      if (stream.match(/^[a-zA-Z_][\w]*/)) {
        const cur = stream.current();
        if (keywords.has(cur)) return "keyword";
        if (builtins.has(cur)) return "builtin";
        if (atoms.has(cur)) return "atom";
        return "variable";
      }

      // Anything else
      stream.next();
      return null;
    }
  };
});

// Optional MIME registration
CodeMirror.defineMIME("text/x-wpp", "wpp");

});
