using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using WPlusPlus.AST;

namespace WPlusPlus
{
    public delegate Task<object> FunctionObject(List<object> args);
public delegate Task<object> AsyncFunctionObject(List<object> args);


    public class Interpreter
    {
        private readonly Dictionary<string, (object Value, bool IsConst)> variables;

        // Default constructor: creates a fresh scope
        public Interpreter()
        {
            variables = new Dictionary<string, (object, bool)>();
        }

        // Overloaded constructor: clones parent scope
        public Interpreter(Dictionary<string, (object Value, bool IsConst)> parentScope)
        {
            variables = new Dictionary<string, (object, bool)>(parentScope);
        }

        public async Task<object> Evaluate(Node node)
        {
            switch (node)
            {
                case NumberNode num:
                    return double.Parse(num.Value);
                case StringNode str:
                    return str.Value;


                case IdentifierNode id:
                    if (variables.TryGetValue(id.Name, out var val))
                    {
                        if (val.Value is double d) return d;
                        if (val.Value is string s) return s;
                        if (val.Value is FunctionObject or AsyncFunctionObject) return val.Value;
                        if (val.Value is null) return null; // ✅ NEW: handle null

                        throw new Exception("Unsupported value type.");
                    }
                    throw new Exception($"Undefined variable: {id.Name}");



                case VariableDeclarationNode decl:
                    if (variables.ContainsKey(decl.Name))
                        throw new Exception($"Variable '{decl.Name}' already declared.");
                    var declaredValue = await Evaluate(decl.Value);
                    variables[decl.Name] = (declaredValue, decl.IsConstant);
                    return declaredValue;

                case AssignmentNode assign:
                    if (variables.TryGetValue(assign.Identifier.Name, out var existing) && existing.IsConst)
                        throw new Exception($"Cannot assign to constant: {assign.Identifier.Name}");
                    var newVal = await Evaluate(assign.Value);
                    variables[assign.Identifier.Name] = (newVal, false);
                    return newVal;

                case PrintNode print:
                    var result = await Evaluate(print.Expression);
                    Console.WriteLine(result);
                    return result;


                case LambdaNode lambda:
                    FunctionObject func = async (args) =>
{
    var local = new Interpreter();
    for (int i = 0; i < lambda.Parameters.Count; i++)
    {
        local.variables[lambda.Parameters[i]] = (args[i], false);
    }

    try
    {
        var result = await local.Evaluate(lambda.Body);
        return result;
    }
    catch (ReturnException retEx)
    {
        return retEx.Value;
    }
};

                    return func;


                case AsyncLambdaNode asyncLambda:
    AsyncFunctionObject asyncFunc = async (args) =>
    {
        var local = new Interpreter();
        for (int i = 0; i < asyncLambda.Parameters.Count; i++)
        {
            local.variables[asyncLambda.Parameters[i]] = (args[i], false);
        }

        try
        {
            var result = await local.Evaluate(asyncLambda.Body);
            return result;
        }
        catch (ReturnException retEx)
        {
            return retEx.Value;
        }
    };
    return asyncFunc;



                case AwaitNode awaitNode:
                    var taskObj = await Evaluate(awaitNode.Expression);

                    if (taskObj is Task task)
                    {
                        await task.ConfigureAwait(false); // Await it
                        var resultProperty = task.GetType().GetProperty("Result");
                        if (resultProperty != null)
                            return resultProperty.GetValue(task);
                    }

                    throw new Exception("Expected a task in await expression.");


                case CallNode call:
                    var callee = await Evaluate(call.Callee);

                    var args = new List<object>();
foreach (var arg in call.Arguments)
    args.Add(await Evaluate(arg));


                    return callee switch
                    {
                        FunctionObject f => await f(args),
                        AsyncFunctionObject af => af(args), // ✅ Return Task<double>, not awaited yet
                        _ => throw new Exception("Trying to call a non-function value.")
                    };

                case ReturnNode ret:
                    var returnVal = await Evaluate(ret.Expression);
                    throw new ReturnException(returnVal);
                case BooleanNode b:
                    return b.Value ? 1.0 : 0.0; // or true/false if supporting real booleans

                case NullNode:
                    return null; // or 0.0 or NaN depending on how you want to handle nulls
                case UnaryExpressionNode unary:
                    var operand = await Evaluate(unary.Operand);
                    return unary.Operator switch
                    {
                        "!" => Convert.ToDouble((double)operand == 0), // Treat nonzero as true
                        _ => throw new Exception("Unknown unary operator: " + unary.Operator)
                    };
                case TryCatchNode tryCatch:
                    try
                    {
                        return await Evaluate(tryCatch.TryBlock);
                    }
                    catch (Exception ex)
                    {
                        var local = new Interpreter();
                        local.variables[tryCatch.CatchVariable] = (0.0, false); // you can store the message if needed
                        Console.WriteLine($"[TRY/CATCH] Exception caught: {ex.Message}");
                        return await local.Evaluate(tryCatch.CatchBlock);
                    }
                case ThrowNode throwNode:
                    var exValue = await Evaluate(throwNode.Expression);
                    throw new Exception(exValue?.ToString() ?? "Unknown error");
                case BinaryExpressionNode bin when bin.Operator == "??":
                    {
                        var left = await Evaluate(bin.Left);
                        var right = await Evaluate(bin.Right);
                        return left == null ? right : left;
                    }


                case ForNode forNode:
                    {
                        // Evaluate the initializer
                        await Evaluate(forNode.Initializer);

                        while (true)
                        {
                            var cond = (double)await Evaluate(forNode.Condition);
                            if (cond == 0) break;

                            try
                            {
                                await Evaluate(forNode.Body);
                            }
                            catch (ContinueException)
                            {
                                // just continue to increment
                            }
                            catch (BreakException)
                            {
                                break;
                            }

                            await Evaluate(forNode.Increment);
                        }

                        return 0.0;
                    }
                case SwitchNode sw:
                    {
                        var switchValue = await Evaluate(sw.Expression);

                        foreach (var (caseValue, caseBody) in sw.Cases)
                        {
                            if (caseValue == null || (double)await Evaluate(caseValue) == (double)switchValue)
                            {
                                try
                                {
                                    foreach (var stmt in caseBody)
                                        await Evaluate(stmt);
                                }
                                catch (BreakException)
                                {
                                    break;
                                }

                                break; // after executing the matching case
                            }
                        }

                        // Run default only if no case matched
                        bool matched = false;
                        foreach (var (caseValue, _) in sw.Cases)
                        {
                            if ((double)await Evaluate(caseValue) == (double)switchValue)
                            {
                                matched = true;
                                break;
                            }
                        }

                        if (!matched)
                        {
                            try
                            {
                                foreach (var stmt in sw.DefaultBody)
                                    await Evaluate(stmt);
                            }
                            catch (BreakException)
                            {
                                // Ignore break in default block
                            }
                        }

                        return 0;
                    }
                case ImportNode importNode:
                    {
                        if (!File.Exists(importNode.Path))
                            throw new Exception($"❌ Import file '{importNode.Path}' not found.");

                        string code = File.ReadAllText(importNode.Path);
                        var tokens = Lexer.Tokenize(code);
                        var parser = new Parser(tokens);
                        var importInterpreter = new Interpreter(this.variables); // share scope

                        while (parser.HasMore())
                        {
                            var importNodeEval = parser.Parse();
                            await importInterpreter.Evaluate(importNodeEval);
                        }

                        return null;
                    }












                case BlockNode block:
                    object last = null;
                    foreach (var stmt in block.Statements)
                        last = await Evaluate(stmt);
                    return last;


                case IfElseNode ifelse:
                    var condition = (double)await Evaluate(ifelse.Condition);
                    if (condition != 0)
                        return await Evaluate(ifelse.IfBody);
                    else if (ifelse.ElseBody != null)
                        return await Evaluate(ifelse.ElseBody);
                    return 0;

                case WhileNode loop:
                    while ((double)await Evaluate(loop.Condition) != 0)
                    {
                        try { await Evaluate(loop.Body); }
                        catch (ContinueException) { continue; }
                        catch (BreakException) { break; }
                    }
                    return 0;

                case BreakNode:
                    throw new BreakException();

                case ContinueNode:
                    throw new ContinueException();

                case BinaryExpressionNode bin:
                    return await EvaluateBinary(bin);


                default:
                    throw new Exception("Unknown AST node");
            }
        }

        private async Task<double> EvaluateBinary(BinaryExpressionNode bin)
        {
            var left = await Evaluate(bin.Left);
            var right = await Evaluate(bin.Right);

            // String concatenation for '+'
            if (bin.Operator == "+" && (left is string || right is string))
            {
                Console.WriteLine(left.ToString() + right.ToString());
                return double.NaN; // since we're printing the result, return NaN
            }

            // Equality/Inequality for strings
            if (bin.Operator is "==" or "!=" && (left is string || right is string))
            {
                bool result = bin.Operator == "==" ? Equals(left, right) : !Equals(left, right);
                return result ? 1 : 0;
            }

            // Prevent math ops on strings
            if (left is string || right is string)
                throw new Exception("Cannot apply arithmetic operators to strings.");

            var leftNum = Convert.ToDouble(left);
            var rightNum = Convert.ToDouble(right);

            return bin.Operator switch
            {
                "+" => leftNum + rightNum,
                "-" => leftNum - rightNum,
                "*" => leftNum * rightNum,
                "/" => rightNum == 0 ? throw new DivideByZeroException() : leftNum / rightNum,
                ">" => leftNum > rightNum ? 1 : 0,
                "<" => leftNum < rightNum ? 1 : 0,
                ">=" => leftNum >= rightNum ? 1 : 0,
                "<=" => leftNum <= rightNum ? 1 : 0,
                "==" => leftNum == rightNum ? 1 : 0,
                "!=" => leftNum != rightNum ? 1 : 0,
                "&&" => (leftNum != 0 && rightNum != 0) ? 1 : 0,
                "||" => (leftNum != 0 || rightNum != 0) ? 1 : 0,
                _ => throw new Exception($"Unknown operator: {bin.Operator}")
            };

        }










        public class ReturnException : Exception
        {
            public object Value { get; }

            public ReturnException(object value)
            {
                Value = value;
            }
        }


        public class BreakException : Exception { }
        public class ContinueException : Exception { }
    }
}

