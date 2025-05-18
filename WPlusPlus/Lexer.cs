using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;

namespace WPlusPlus
{
    public enum TokenType
    {
        Identifier,
        Number,
        Operator,
        Symbol,
        Keyword,
        Unknown,
        Comment,
            String
            
    }

    public class Token
    {
        public TokenType Type { get; set; }
        public string Value { get; set; }

        public Token(TokenType type, string value)
        {
            Type = type;
            Value = value;
        }
    }

    public static class Lexer
    {
        public static List<Token> Tokenize(string input)
        {
            var tokens = new List<Token>();

            // ✅ Use List instead of Dictionary to allow multiple Keyword patterns
            var patterns = new List<(TokenType Type, string Pattern)>
{
    (TokenType.Comment, @"//.*"),

    // Keywords
    (TokenType.Keyword, @"\bif\b"),
    (TokenType.Keyword, @"\belse\b"),
    (TokenType.Keyword, @"\blet\b"),
    (TokenType.Keyword, @"\bconst\b"),
    (TokenType.Keyword, @"\bwhile\b"),
    (TokenType.Keyword, @"\bbreak\b"),
    (TokenType.Keyword, @"\bcontinue\b"),
    (TokenType.Keyword, @"\breturn\b"),
    (TokenType.Keyword, @"\bprint\b"),
    (TokenType.Keyword, @"\basync\b"),
    (TokenType.Keyword, @"\bawait\b"),
    (TokenType.Keyword, @"\btrue\b"),     // ✅ Add this
    (TokenType.Keyword, @"\bfalse\b"),    // ✅ And this
    (TokenType.Keyword, @"\bnull\b"),     // ✅ And this
    (TokenType.Keyword, @"\btry\b"),
    (TokenType.Keyword, @"\bcatch\b"),
    (TokenType.Keyword, @"\bthrow\b"),
    (TokenType.Keyword, @"\bfor\b"),
(TokenType.Keyword, @"\bswitch\b"),
(TokenType.Keyword, @"\bcase\b"),
(TokenType.Keyword, @"\bimport\b"),
(TokenType.Keyword, @"\bdefault\b"),
(TokenType.Keyword, @"\bentity\b"),
(TokenType.Keyword, @"\binherits\b"),
(TokenType.Keyword, @"\bdisown\b"),
(TokenType.Keyword, @"\bbirth\b"),
(TokenType.Keyword, @"\bvanish\b"),
(TokenType.Keyword, @"\bme\b"),
(TokenType.Keyword, @"\bancestor\b"),
(TokenType.Keyword, @"\bnew\b"),
(TokenType.Keyword, @"\balters\b"),







    // Identifiers and literals
    (TokenType.String, @"""[^""]*"""),
    (TokenType.Identifier, @"\b[a-zA-Z_][a-zA-Z0-9_]*\b"),
    (TokenType.Number, @"\b\d+(\.\d+)?\b"),

    // Operators — NOTE: '=>' must go before '='
        (TokenType.Operator, @"\?\?|=>|==|!=|<=|>=|&&|\|\||[+\-*/=<>!]"),

    // Symbols
    (TokenType.Symbol, @"[().,;:{}]")
};





            var combinedPattern = string.Join("|", patterns.ConvertAll(p => $"(?<{p.Type}>{p.Pattern})"));
            var regex = new Regex(combinedPattern);

            var matches = regex.Matches(input);
            foreach (Match match in matches)
            {
                bool matched = false;
                foreach (var groupName in regex.GetGroupNames())
{
    if (groupName == "0") continue; // skip whole match

    var group = match.Groups[groupName];
    if (group.Success && Enum.TryParse(groupName, out TokenType type))
    {
        tokens.Add(new Token(type, group.Value));
        matched = true;
        break;
    }
}



                if (!matched && !string.IsNullOrWhiteSpace(match.Value))
                {
                    throw new Exception($"[❌ Lexer Error] Unrecognized token: '{match.Value}' at index {match.Index}");
                }


            }




            return tokens;
        }
    }
}

