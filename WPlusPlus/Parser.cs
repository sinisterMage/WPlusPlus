using System;
using System.Collections.Generic;
using WPlusPlus.AST;

namespace WPlusPlus
{
    public class Parser
    {
        private readonly List<Token> tokens;
        private static HashSet<string> importedFiles = new();

        private int position;

        public Parser(List<Token> tokens)
        {
            this.tokens = tokens;
            position = 0;
        }

        public bool HasMore()
        {
            return position < tokens.Count;
        }

        public Node Parse()
        {
            return ParseStatement();
        }

        private Node ParseStatement()
        {
            if (Match(TokenType.Keyword))
            {
                var keyword = Peek().Value;

                switch (keyword)
                {
                    case "print":
                        Advance();
                        var printExpr = ParseExpression();
                        Console.WriteLine($"[DEBUG] Expecting semicolon after: {Peek()?.Value}");
                        Expect(";");
                        return new PrintNode(printExpr);

                    case "if":
                        Advance();
                        Expect("(");
                        var condition = ParseExpression();
                        Expect(")");
                        var ifBody = ParseOptionalBlockOrStatement();
                        Node elseBody = null;

                        if (Match(TokenType.Keyword) && Peek().Value == "else")
                        {
                            Advance();
                            elseBody = ParseOptionalBlockOrStatement();
                        }


                        return new IfElseNode(condition, ifBody, elseBody);

                    case "let":
                    case "const":
                        bool isConst = keyword == "const";
                        Advance();

                        if (!Match(TokenType.Identifier))
                            throw new Exception("Expected variable name");

                        var name = Advance().Value;

                        if (!Match(TokenType.Operator) || Peek().Value != "=")
                            throw new Exception("Expected '=' after variable name");

                        Advance();
                        var value = ParseExpression();
                        Console.WriteLine($"[DEBUG] Expecting semicolon after: {Peek()?.Value}");
                        Expect(";");
                        return new VariableDeclarationNode(name, value, isConst);

                    case "while":
                        Advance();
                        Expect("(");
                        var loopCond = ParseExpression();
                        Expect(")");
                        return new WhileNode(loopCond, ParseBlock());

                    case "break":
                        Advance();
                        Console.WriteLine($"[DEBUG] Expecting semicolon after: {Peek()?.Value}");
                        Expect(";");
                        return new BreakNode();

                    case "continue":
                        Advance();
                        Console.WriteLine($"[DEBUG] Expecting semicolon after: {Peek()?.Value}");
                        Expect(";");
                        return new ContinueNode();

                    case "return":
                        Advance();
                        var returnExpr = ParseExpression();
                        Console.WriteLine($"[DEBUG] Expecting semicolon after: {Peek()?.Value}");
                        Expect(";");
                        return new ReturnNode(returnExpr);

                    case "true":
                        Advance();
                        return new BooleanNode(true);

                    case "false":
                        Advance();
                        return new BooleanNode(false);

                    case "null":
                        Advance();
                        return new NullNode();

                    case "try":
                        Advance(); // 'try'
                        var tryBlock = ParseBlock();
                        ExpectKeyword("catch");
                        Expect("(");
                        if (!Match(TokenType.Identifier))
                            throw new Exception("Expected identifier in catch(...)");
                        string catchVar = Advance().Value;
                        Expect(")");
                        var catchBlock = ParseBlock();
                        return new TryCatchNode(tryBlock, catchVar, catchBlock);

                    case "throw":
                        Advance(); // consume 'throw'
                        var throwExpr = ParseExpression();
                        Console.WriteLine("[DEBUG] Expecting semicolon after: " + Peek()?.Value);
                        Expect(";");
                        return new ThrowNode(throwExpr);
                    case "for":
                        {
                            Advance(); // consume 'for'
                            Expect("(");

                            Node initializer;

                            if (Peek().Value == "let" || Peek().Value == "const")
                            {
                                Console.WriteLine("[DEBUG] Detected variable declaration inside for loop");
                                initializer = ParseDeclarationOnly(); // does NOT consume semicolon
                                Expect(";"); // consume it explicitly here
                            }
                            else
                            {
                                initializer = ParseExpression();
                                Expect(";");
                            }

                            var forCondition = ParseExpression();
                            Expect(";");

                            var increment = ParseExpression(); // use ParseExpression, not ParseStatement
                            Expect(")");

                            var body = ParseBlock();
                            return new ForNode(initializer, forCondition, increment, body);
                        }
                    case "switch":
                        {
                            Advance(); // 'switch'
                            Expect("(");
                            var switchExpr = ParseExpression();
                            Expect(")");

                            Expect("{");
                            var cases = new List<(Node, List<Node>)>();
                            List<Node> defaultBody = null;

                            while (!(Match(TokenType.Symbol) && Peek().Value == "}"))
                            {
                                if (Match(TokenType.Keyword) && Peek().Value == "case")
                                {
                                    Advance();
                                    var caseValue = ParseExpression();
                                    Expect(":");

                                    var body = new List<Node>();
                                    while (!(Match(TokenType.Keyword) && (Peek().Value == "case" || Peek().Value == "default")) &&
                                           !(Match(TokenType.Symbol) && Peek().Value == "}"))
                                    {
                                        body.Add(ParseStatement());
                                    }

                                    cases.Add((caseValue, body));
                                }
                                else if (Match(TokenType.Keyword) && Peek().Value == "default")
                                {
                                    Advance();
                                    Expect(":");

                                    defaultBody = new List<Node>();
                                    while (!(Match(TokenType.Symbol) && Peek().Value == "}"))
                                    {
                                        defaultBody.Add(ParseStatement());
                                    }
                                }
                                else
                                {
                                    throw new Exception("Unexpected token inside switch block");
                                }
                            }

                            Expect("}");

                            return new SwitchNode(switchExpr, cases, defaultBody);
                        }
                    case "import":
                        {
                            Advance(); // consume 'import'
                            if (!Match(TokenType.String))
                                throw new Exception("Expected string path in import");

                            var path = Advance().Value.Trim('"');
                            Expect(";");

                            if (importedFiles.Contains(path))
                            {
                                Console.WriteLine($"[DEBUG] Skipping already imported: {path}");
                                return new NoOpNode();
                            }

                            importedFiles.Add(path);

                            if (!File.Exists(path))
                                throw new Exception($"Imported file not found: {path}");

                            var code = File.ReadAllText(path);
                            var newTokens = Lexer.Tokenize(code);
                            var newParser = new Parser(newTokens);

                            var nodes = new List<Node>();
                            while (newParser.HasMore())
                            {
                                nodes.Add(newParser.Parse());
                            }

                            return new BlockNode(nodes); // execute imported code in block scope
                        }











                }
            }

            if (Match(TokenType.Identifier) && LookAhead()?.Type == TokenType.Operator && LookAhead()?.Value == "=")
            {
                var identifier = new IdentifierNode(Advance().Value);
                Advance(); // consume '='
                var value = ParseExpression();
                Expect(";");
                return new AssignmentNode(identifier, value);
            }

            // Fallback: expression statement
            try
            {
                var expr = ParseExpression();
                Console.WriteLine($"[DEBUG] Expecting semicolon after: {Peek()?.Value}");
                Expect(";"); // Require semicolon to disambiguate
                return expr;
            }
            catch
            {
                throw new Exception("Unrecognized statement starting at: " + Peek()?.Value);
            }
        }


        private Node ParseBlock()
        {
            Console.WriteLine($"[DEBUG] Entering block. Peek: {Peek()?.Value}, Type: {Peek()?.Type}");

            if (!Match(TokenType.Symbol) || Peek().Value != "{")
                throw new Exception($"Expected '{{' to start block but found: {Peek()?.Value}");

            Advance();
            var statements = new List<Node>();

            while (!(Match(TokenType.Symbol) && Peek().Value == "}"))
            {
                statements.Add(ParseStatement());
            }

            Expect("}");
            return new BlockNode(statements);
        }


        private Node ParseExpression()
        {
            return ParseAssignment();
        }

        private Node ParseAssignment()
        {
            var left = ParseBinaryExpression(ParsePrimary(), 0);

            // Handle assignment: i = i + 1
            if (Match(TokenType.Operator) && Peek().Value == "=")
            {
                Advance(); // consume '='
                var right = ParseAssignment(); // recursive to support i = j = 3;
                if (left is IdentifierNode id)
                {
                    return new AssignmentNode(id, right);
                }
                else
                {
                    throw new Exception("Invalid assignment target");
                }
            }

            return left;
        }


        private Node ParseBinaryExpression(Node left, int parentPrecedence)
        {
            while (Match(TokenType.Operator) && GetPrecedence(Peek().Value) > parentPrecedence)
            {
                var op = Advance().Value;
                var precedence = GetPrecedence(op);

                // 🔥 FIX: ParseExpression instead of ParsePrimary
                var right = ParsePrimary();

                left = new BinaryExpressionNode(left, op, right);
            }

            return left;
        }






        private int GetPrecedence(string op) => op switch
        {
            "||" => 1,
            "&&" => 2,
            "??" => 2, // ✅ add this
            "==" or "!=" => 3,
            ">" or "<" or ">=" or "<=" => 4,
            "+" or "-" => 5,
            "*" or "/" => 6,
            _ => 0
        };

        private void ExpectKeyword(string keyword)
        {
            if (!Match(TokenType.Keyword) || Peek().Value != keyword)
                throw new Exception($"Expected keyword '{keyword}'");
            Advance();
        }




        private Node ParsePrimary()
        {
            Console.WriteLine($"[DEBUG] Peek: {Peek()?.Value}, Type: {Peek()?.Type}");

            // Unary: !expr
            if (Match(TokenType.Operator) && Peek().Value == "!")
            {
                var op = Advance().Value;
                var operand = ParseExpression(); // allows !(a && b)
                return new UnaryExpressionNode(op, operand);
            }

            // Number literal
            if (Match(TokenType.Number))
                return new NumberNode(Advance().Value);

            // String literal
            if (Match(TokenType.String))
                return new StringNode(Advance().Value);

            // Identifier or function call
            if (Match(TokenType.Identifier))
            {
                var idToken = Advance();
                var expr = new IdentifierNode(idToken.Value);

                // Function call: name(args)
                if (Match(TokenType.Symbol) && Peek().Value == "(")
                {
                    Advance(); // consume '('
                    var arguments = new List<Node>();

                    if (!(Match(TokenType.Symbol) && Peek().Value == ")"))
                    {
                        do
                        {
                            arguments.Add(ParseExpression());
                        } while (Match(TokenType.Symbol) && Peek().Value == "," && Advance() != null);
                    }

                    Expect(")");
                    return new CallNode(expr, arguments);
                }

                return expr;
            }

            // Boolean literals
            if (Match(TokenType.Keyword) && (Peek().Value == "true" || Peek().Value == "false"))
            {
                var value = Advance().Value == "true" ? "1" : "0";
                return new NumberNode(value);
            }
            // Null literal
            if (Match(TokenType.Keyword) && Peek().Value == "null")
            {
                Advance();
                return new NullNode();
            }


            // Await expression
            if (Match(TokenType.Keyword) && Peek().Value == "await")
            {
                Advance();
                var expr = ParsePrimary();
                return new AwaitNode(expr);
            }
            // Support: async (...) => ...
            if (Match(TokenType.Keyword) && Peek().Value == "async")
            {
                Advance(); // consume 'async'
                if (Match(TokenType.Symbol) && Peek().Value == "(")
                {
                    Console.WriteLine($"[DEBUG] Lambda detected. IsAsync = true");
                    return ParseLambda(true);
                }
                else
                {
                    throw new Exception("Expected '(' after 'async'");
                }
            }


            // Lambda: (x, y) => ...
            if (Match(TokenType.Symbol) && Peek().Value == "(")
            {
                int temp = position;
                int parenCount = 1;
                bool isLambda = false;

                while (++temp < tokens.Count)
                {
                    if (tokens[temp].Value == "(") parenCount++;
                    if (tokens[temp].Value == ")") parenCount--;
                    if (parenCount == 0)
                    {
                        if (temp + 1 < tokens.Count && tokens[temp + 1].Value == "=>")
                        {
                            isLambda = true;
                        }
                        break;
                    }
                }

                if (isLambda)
                {
                    bool isAsync = (position > 0 && tokens[position - 1].Value == "async");
                    Console.WriteLine($"[DEBUG] Lambda detected. IsAsync = {isAsync}");
                    return ParseLambda(isAsync);
                }


                // Otherwise: grouped expression
                Advance(); // consume '('
                var expr = ParseExpression();
                Expect(")");
                return expr;
            }

            throw new Exception("Unexpected token: " + Peek()?.Value);
        }


        private Node ParseDeclarationOnly()
        {
            var keyword = Advance().Value;
            bool isConst = keyword == "const";

            if (!Match(TokenType.Identifier))
                throw new Exception("Expected variable name");

            var name = Advance().Value;

            if (!Match(TokenType.Operator) || Peek().Value != "=")
                throw new Exception("Expected '=' after variable name");

            Advance(); // consume '='
            var value = ParseExpression();

            return new VariableDeclarationNode(name, value, isConst);
        }




        private Node ParseLambda(bool async)
        {
            Expect("(");
            var parameters = new List<string>();

            while (!Match(TokenType.Symbol) || Peek().Value != ")")
            {
                if (!Match(TokenType.Identifier))
                    throw new Exception("Expected parameter name");

                parameters.Add(Advance().Value);

                if (Match(TokenType.Symbol))
                {
                    if (Peek().Value == ",")
                    {
                        Advance(); // consume comma
                        continue;
                    }
                    else if (Peek().Value == ")")
                    {
                        break; // done
                    }
                }

                throw new Exception($"Expected ',' or ')' in lambda parameter list but found: {Peek()?.Value}, Type: {Peek()?.Type}");
            }

            Expect(")");

            if (!Match(TokenType.Operator) || Peek().Value != "=>")
                throw new Exception("Expected '=>' in lambda");

            Advance(); // consume =>

            Node body = Match(TokenType.Symbol) && Peek().Value == "{"
                ? ParseBlock()
                : ParseBinaryExpression(ParsePrimary(), 0);

            return async
                ? new AsyncLambdaNode(parameters, body)
                : new LambdaNode(parameters, body);
        }



        private Node ParseOptionalBlockOrStatement()
        {
            if (Match(TokenType.Symbol) && Peek().Value == "{")
                return ParseBlock();
            else
                return ParseStatement(); // allow one-liners like `continue;`
        }



        private Token LookAheadUntil(string target)
        {
            int temp = position;
            while (temp < tokens.Count)
            {
                if (tokens[temp].Value == target)
                    return tokens[temp];
                temp++;
            }
            return null;
        }








        private void Expect(string symbol)
        {
            Console.WriteLine($"[DEBUG] Before final ')' expect: {Peek()?.Value}");
            if (!Match(TokenType.Symbol) || Peek().Value != symbol)

                throw new Exception($"Expected '{symbol}'");
            Advance();
        }

        private bool Match(TokenType type)
        {
            return Peek()?.Type == type;
        }

        public Token Peek()
{
    while (position < tokens.Count && tokens[position].Type == TokenType.Comment)
        position++;
    return position < tokens.Count ? tokens[position] : null;
}

public Token Advance()
{
    var token = Peek();
    position++;
    return token;
}


        private Token LookAhead(int offset = 1)
        {
            return (position + offset < tokens.Count) ? tokens[position + offset] : null;
        }
    }
}
