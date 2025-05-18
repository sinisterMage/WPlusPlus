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
    while (position < tokens.Count && tokens[position].Type == TokenType.Comment)
        position++;
    return position < tokens.Count;
}


        public Node Parse()
{
    var statements = new List<Node>();

    while (HasMore())
    {
        try
        {
            var stmt = ParseStatement();
            if (stmt != null)
                statements.Add(stmt);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[ERROR] While parsing statement at token: {Peek()?.Value ?? "EOF"}");
            throw;
        }
    }

    return new BlockNode(statements);
}



        public Node ParseStatement()
        {
            if (Peek() == null)
    throw new Exception("Unexpected end of input");

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
                    case "entity":
                        {
                            Advance(); // 'entity'

                            if (!Match(TokenType.Identifier))
                                throw new Exception("Expected entity name");

                            var entityName = Advance().Value;
                            string? parent = null;
                            bool disowns = false;

                            if (Match(TokenType.Keyword) && Peek().Value == "inherits")
                            {
                                Advance();
                                if (!Match(TokenType.Identifier))
                                    throw new Exception("Expected parent entity name");
                                parent = Advance().Value;
                            }

                            if (Match(TokenType.Keyword) && Peek().Value == "disown")
                            {
                                Advance();
                                disowns = true;
                            }

                            Expect("{");

                            var definition = new EntityDefinition
                            {
                                Name = entityName,
                                Parent = parent,
                                Disowns = disowns,
                                Methods = new Dictionary<string, MethodNode>()
                            };

                            while (!(Match(TokenType.Symbol) && Peek().Value == "}"))
                            {
                                // ✅ Handle `disown` keyword inside entity body
                                if (Match(TokenType.Keyword) && Peek().Value == "disown")
                                {
                                    Advance(); // consume 'disown'
                                    disowns = true;
                                    continue;  // move to next token without processing it as a statement
                                }

                                Node stmt;

                                if (Match(TokenType.Identifier) && LookAhead()?.Value == "=>")
                                {
                                    stmt = ParseEntityMethod();
                                }
                                else
                                {
                                    stmt = ParseStatement();
                                }

                                if (stmt is MethodNode method)
                                {
                                    definition.Methods[method.Name] = method;
                                }
                                else
                                {
                                    Console.WriteLine($"[WARN] Non-method ignored in entity '{entityName}'");
                                }
                            }

                            Expect("}");

                            return new EntityNode(entityName, parent, disowns, new List<Node>(definition.Methods.Values));
                        }


                    case "alters":
                        {
                            Advance(); // consume 'alters'

                            if (!Match(TokenType.Identifier))
                                throw new Exception("Expected child entity name");

                            var child = Advance().Value;

                            if (!Match(TokenType.Keyword) || Peek().Value != "alters")
                                throw new Exception("Expected 'alters' again in 'Child alters Parent'");

                            Advance(); // consume 2nd 'alters'

                            if (!Match(TokenType.Identifier))
                                throw new Exception("Expected parent entity name");

                            var parent = Advance().Value;

                            Expect("{");
                            var methodList = new List<Node>();

                            while (!(Match(TokenType.Symbol) && Peek().Value == "}"))
                            {
                                // ✅ Allow method definitions like: whoami => { ... }
                                if (Match(TokenType.Identifier) && LookAhead()?.Value == "=>")
                                {
                                    methodList.Add(ParseEntityMethod());
                                }
                                else
                                {
                                    methodList.Add(ParseStatement());
                                }
                            }

                            Expect("}");

                            return new AltersNode(child, parent, methodList);
                        }


















                }
                // Add this right after the keyword switch block and before the `if (Match(TokenType.Identifier))`
                if (Match(TokenType.Keyword) && Peek().Value == "await")
                {
                    var expr = ParseExpression(); // ✅ this will consume `await who()`
                    Expect(";");                  // ✅ require semicolon like other expressions
                    return expr;
                }


            }

            if (Match(TokenType.Identifier))
            {
                try
                {
                    // Always start with a full expression (not just a single identifier)
                    var expr = ParseExpression(); // this already supports dog.speak()

                    // Make sure a semicolon follows it
                    if (!Match(TokenType.Symbol) || Peek().Value != ";")
                    {
                        Console.WriteLine($"[DEBUG] Expected ';' after expression but found: {Peek()?.Value}");
                        throw new Exception("Expected ';'");
                    }

                    Advance(); // consume the semicolon
                    return expr;
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[DEBUG] Token before error: {Peek()?.Value} (type: {Peek()?.Type})");
                    throw new Exception("Unrecognized statement starting at identifier: " + Peek()?.Value + "\nInner: " + ex.Message);
                }
            }







            // Fallback: expression statement
            try
            {
                var expr = ParseExpression();

                if (!Match(TokenType.Symbol) || Peek().Value != ";")
                {
                    Console.WriteLine($"[DEBUG] After expression, expected ';' but found: {Peek()?.Value}");
                    throw new Exception("Expected ';' after expression");
                }


                Advance(); // Consume the semicolon

                return expr;



            }
            catch (Exception ex)
{
    var peek = Peek();
    var message = peek != null
        ? $"Unrecognized statement starting at identifier: {peek.Value}\nInner: {ex.Message}"
        : "Unexpected end of input";

    throw new Exception(message);
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
    var left = ParseBinaryExpression(ParseUnary(), 0);

    // Support chaining like: dog.speak().another()
    while (Match(TokenType.Symbol) && Peek().Value == ".")
    {
        Advance(); // consume '.'

        if (!Match(TokenType.Identifier))
            throw new Exception("Expected property or method name after '.'");

        var memberName = Advance().Value;

        if (Match(TokenType.Symbol) && Peek().Value == "(")
        {
            Advance(); // consume '('
            var args = new List<Node>();

            if (!(Match(TokenType.Symbol) && Peek().Value == ")"))
            {
                do
                {
                    args.Add(ParseExpression());
                } while (Match(TokenType.Symbol) && Peek().Value == "," && Advance() != null);
            }

            Expect(")");

            left = new CallNode(new MemberAccessNode(left, memberName), args);
        }
        else
        {
            left = new MemberAccessNode(left, memberName);
        }
    }

    // Handle assignment: a.b = 3
    if (Match(TokenType.Operator) && Peek().Value == "=")
    {
        Advance(); // consume '='
        var right = ParseAssignment(); // recursive
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
                var right = ParseUnary();

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
                Node expr = new IdentifierNode(idToken.Value);

                while (true)
                {
                    if (Match(TokenType.Symbol) && Peek().Value == "(")
                    {
                        Advance(); // consume '('
                        var args = new List<Node>();

                        if (!(Match(TokenType.Symbol) && Peek().Value == ")"))
                        {
                            while (true)
                            {
                                if (Peek() == null || Peek().Value == ")")
                                    break;

                                args.Add(ParseExpression());

                                if (Match(TokenType.Symbol) && Peek().Value == ",")
                                {
                                    Advance(); // consume comma
                                    if (Peek() == null || Peek().Value == ")")
                                        throw new Exception("Trailing comma in argument list is not allowed");
                                }
                                else
                                {
                                    break;
                                }
                            }
                        }


                        Expect(")");
                        expr = new CallNode(expr, args);
                    }
                    else if (Match(TokenType.Symbol) && Peek().Value == ".")
                    {
                        Advance(); // consume '.'

                        if (!Match(TokenType.Identifier))
                            throw new Exception("Expected identifier after '.'");

                        var memberName = Advance().Value;

                        if (Match(TokenType.Symbol) && Peek().Value == "(")
                        {
                            Advance(); // consume '('
                            var args = new List<Node>();

                            if (!(Match(TokenType.Symbol) && Peek().Value == ")"))
                            {
                                while (true)
                                {
                                    if (Peek() == null || Peek().Value == ")")
                                        break;

                                    args.Add(ParseExpression());

                                    if (Match(TokenType.Symbol) && Peek().Value == ",")
                                    {
                                        Advance(); // consume comma
                                        if (Peek() == null || Peek().Value == ")")
                                            throw new Exception("Trailing comma in method call is not allowed");
                                    }
                                    else
                                    {
                                        break;
                                    }
                                }
                            }


                            Expect(")");
                            expr = new CallNode(new MemberAccessNode(expr, memberName), args);
                        }
                        else
                        {
                            expr = new MemberAccessNode(expr, memberName);
                        }
                    }
                    else
                    {
                        break;
                    }
                }

                return expr;
            }


            // Boolean literals
            if (Match(TokenType.Keyword) && (Peek().Value == "true" || Peek().Value == "false"))
            {
                var value = Advance().Value == "true" ? "1" : "0";
                return new NumberNode(value);
            }
            // 🔥 Handle 'me' keyword as MeNode
            if (Match(TokenType.Keyword) && Peek().Value == "me")
            {
                Advance();
                return new MeNode();
            }

            // Null literal
            if (Match(TokenType.Keyword) && Peek().Value == "null")
            {
                Advance();
                return new NullNode();
            }
            if (Match(TokenType.Keyword) && Peek().Value == "new")
            {
                Advance(); // consume 'new'

                Expect("(");

                if (!Match(TokenType.Identifier))
                    throw new Exception("Expected entity name inside 'new(...)'");

                var entityName = Advance().Value;

                Expect(")"); // Close the 'new(...)'

                return new NewNode(entityName);
            }

            // 🔥 Add this just above other keyword checks like 'await'
            if (Match(TokenType.Keyword) && Peek().Value == "ancestor")
            {
                Advance(); // consume 'ancestor'

                // ✅ Support optional dot before method name
                if (Match(TokenType.Symbol) && Peek().Value == ".")
                {
                    Advance(); // consume '.'
                }

                if (!Match(TokenType.Identifier))
                    throw new Exception("Expected method name after 'ancestor'");

                var methodName = Advance().Value;

                Expect("(");
                var arguments = new List<Node>();
                while (!(Match(TokenType.Symbol) && Peek().Value == ")"))
                {
                    arguments.Add(ParseExpression());

                    if (Match(TokenType.Symbol) && Peek().Value == ",")
                        Advance(); // consume comma
                }
                Expect(")");

                return new AncestorCallNode(methodName, arguments);
            }






            // Await expression
            if (Match(TokenType.Keyword) && Peek().Value == "await")
            {
                Advance();
                Console.WriteLine($"[DEBUG] Awaiting expression starting at: {Peek()?.Value}");
                var expr = ParseExpression(); // ✅ supports full call: await who()
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
        private Node ParseUnary()
{
    if (Match(TokenType.Operator) && Peek().Value == "!")
    {
        var op = Advance().Value;
        var right = ParseUnary();
        return new UnaryExpressionNode(op, right);
    }



    return ParsePrimary();
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
        private MethodNode ParseEntityMethod()
{
    if (!Match(TokenType.Identifier))
        throw new Exception("Expected method name");

    var methodName = Advance().Value;

    if (!Match(TokenType.Operator) || Peek().Value != "=>")
        throw new Exception($"Expected '=>' after method name '{methodName}'");

    Advance(); // consume '=>'

    Node body;

    if (Match(TokenType.Symbol) && Peek().Value == "{")
    {
        body = ParseBlock(); // ✅ multi-line body (safe)
    }
    else
    {
        // ✅ Force single-line statement to end with ;
        body = ParseStatement();

        if (body == null)
            throw new Exception($"Method '{methodName}' body is null");
    }

    if (body == null)
        throw new Exception($"Parsed method '{methodName}' has null body");

    return new MethodNode(methodName, new List<string>(), body);
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
