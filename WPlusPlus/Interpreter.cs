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
        private Dictionary<string, EntityDefinition> entityTable = new();
        private Dictionary<string, object> currentInstance = null;
        private Dictionary<string, Dictionary<string, MethodNode>> originalMethodTable = new();

private string currentEntity = null;


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
            Console.WriteLine("[INTERPRETER] Evaluating via interpreter");
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
                        if (val.Value is Dictionary<string, object> dict) return dict;
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
                    {
                        var callee = await Evaluate(call.Callee);

                        var args = new List<object>();
                        foreach (var arg in call.Arguments)
                            args.Add(await Evaluate(arg));

                        switch (callee)
                        {
                            case FunctionObject f:
                                return await f(args);

                            case AsyncFunctionObject af:
                                return af(args); // ✅ return Task<object>, do not await

                            case MethodNode method:
                                {
                                    var methodScope = new Interpreter(this.variables)
                                    {
                                        entityTable = this.entityTable,
                                        originalMethodTable = this.originalMethodTable, // ✅ ADD THIS
                                    };

                                    if (call.Callee is MemberAccessNode memberAccess)
                                    {
                                        var target = await Evaluate(memberAccess.Target);
                                        if (target is Dictionary<string, object> instance)
                                        {
                                            methodScope.currentInstance = instance;

                                            if (instance.TryGetValue("__entity__", out var entName) && entName is string entStr)
                                            {
                                                Console.WriteLine("[DEBUG] Setting currentEntity from instance: " + entStr);
                                                methodScope.currentEntity = entStr;
                                            }
                                        }
                                    }
                                    else
                                    {
                                        methodScope.currentInstance = this.currentInstance;
                                        methodScope.currentEntity = this.currentEntity;
                                    }

                                    for (int i = 0; i < method.Parameters.Count; i++)
                                        methodScope.variables[method.Parameters[i]] = (args[i], false);

                                    Console.WriteLine($"[CALL] Executing method '{method.Name}' from entity '{methodScope.currentEntity}'");
                                    return await methodScope.Evaluate(method.Body);
                                }

                            default:
                                throw new Exception("Trying to call a non-function value.");
                        }
                    }




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
                            var importNodeEval = parser.ParseStatement(); // ✅ correct for multiple in loop
                            await importInterpreter.Evaluate(importNodeEval);
                        }

                        return null;
                    }

                case EntityNode entity:
                    {
                        Console.WriteLine($"[DEBUG] Defining entity: {entity.Name}");

                        var definition = new EntityDefinition
                        {
                            Name = entity.Name,
                            Parent = entity.Parent,
                            Disowns = entity.DisownsParent,
                            Methods = new Dictionary<string, MethodNode>()
                        };

                        foreach (var stmt in entity.Body)
                        {
                            if (stmt is MethodNode method)
                            {
                                Console.WriteLine($"[DEBUG] Adding method '{method.Name}' to '{entity.Name}'");
                                definition.Methods[method.Name] = method;
                            }
                        }

                        entityTable[entity.Name] = definition;

                        EntityDefinition walker = definition;
                        while (walker != null)
                        {
                            if (!entityTable.TryGetValue(walker.Name, out var def))
                            {
                                Console.WriteLine($"[WARN] Could not find definition for '{walker.Name}' during backup");
                                break;
                            }

                            // Create backup table for this level if missing
                            if (!originalMethodTable.ContainsKey(walker.Name))
                            {
                                originalMethodTable[walker.Name] = new Dictionary<string, MethodNode>();
                                Console.WriteLine($"[DEBUG] Created backup table for entity: {walker.Name}");
                            }

                            var backup = originalMethodTable[walker.Name];

                            foreach (var method in def.Methods)
                            {
                                if (!backup.ContainsKey(method.Key))
                                {
                                    backup[method.Key] = new MethodNode(
                                        method.Value.Name,
                                        new List<string>(method.Value.Parameters),
                                        method.Value.Body
                                    );
                                    Console.WriteLine($"[DEBUG] Backing up method '{method.Key}' for entity '{def.Name}'");
                                }
                            }

                            if (def.Disowns || def.Parent == null)
                            {
                                Console.WriteLine($"[STOP] Stopping walk from '{def.Name}' due to disown or no parent.");
                                break;
                            }

                            if (!entityTable.TryGetValue(def.Parent, out walker))
                            {
                                Console.WriteLine($"[WARN] Parent '{def.Parent}' not found for '{def.Name}'");
                                break;
                            }
                        }

                        // 🧠 Log the final backup content
                        Console.WriteLine($"[VERIFY] Backup table for '{entity.Name}' contains:");
                        foreach (var kv in originalMethodTable[entity.Name])
                            Console.WriteLine($"  -> {kv.Key}");

                        return null;
                    }



                case AltersNode alters:
                    {
                        Console.WriteLine($"[DEBUG] Altering {alters.TargetAncestor} with child {alters.ChildEntity}");

                        if (!entityTable.TryGetValue(alters.TargetAncestor, out var parent))
                            throw new Exception($"Ancestor entity '{alters.TargetAncestor}' not found");

                        if (!entityTable.TryGetValue(alters.ChildEntity, out var childDef))
                            throw new Exception($"Child entity '{alters.ChildEntity}' not found");

                        if (!originalMethodTable.ContainsKey(childDef.Name))
                        {
                            originalMethodTable[childDef.Name] = new Dictionary<string, MethodNode>();
                            Console.WriteLine($"[DEBUG] Creating backup for child entity: {childDef.Name}");
                        }

                        var childBackup = originalMethodTable[childDef.Name];

                        Console.WriteLine($"[CHECK] Existing backup methods for '{childDef.Name}':");
                        foreach (var kv in childBackup)
                            Console.WriteLine($"  >> {kv.Key}");

                        foreach (var method in alters.AlteredMethods)
                        {
                            if (method is MethodNode m)
                            {
                                if (!childBackup.ContainsKey(m.Name))
                                {
                                    var inherited = GetAllMethods(parent).FirstOrDefault(kv => kv.Key == m.Name).Value;
                                    if (inherited != null)
                                    {
                                        childBackup[m.Name] = new MethodNode(
                                            inherited.Name,
                                            new List<string>(inherited.Parameters),
                                            inherited.Body
                                        );
                                        Console.WriteLine($"[BACKUP] Preserved '{m.Name}' for '{childDef.Name}' from '{parent.Name}'");
                                    }
                                    else
                                    {
                                        Console.WriteLine($"[WARN] Could not find inherited method '{m.Name}' for backup.");
                                    }
                                }

                                Console.WriteLine($"[OVERRIDE] {childDef.Name}.{m.Name} now altered");
                                childDef.Methods[m.Name] = m;
                            }
                        }

                        return null;
                    }






                case NewNode newNode:
                    {
                        if (!entityTable.TryGetValue(newNode.EntityName, out var definition))
                            throw new Exception($"Entity '{newNode.EntityName}' not found");

                        var instance = new Dictionary<string, object>
                        {
                            ["__entity__"] = definition.Name
                        };

                        // Attach methods (not executed now)
                        foreach (var method in GetAllMethods(definition))
                        {
                            instance[method.Key] = method.Value;
                        }

                        return instance;
                    }
                case MeNode:
                    {
                        if (currentInstance == null)
                            throw new Exception("me is not available in this context.");
                        return currentInstance;
                    }
                case AncestorCallNode ancestorCall:
                    {
                        string methodName = ancestorCall.MethodName;
                        Console.WriteLine($"[DEBUG] Starting ancestor call for method: {methodName}");
                        Console.WriteLine($"[DEBUG] Initial currentEntity: {currentEntity}");

                        string lookupEntity = currentEntity;

                        if (!originalMethodTable.TryGetValue(lookupEntity, out var backups))
                        {
                            Console.WriteLine($"[FATAL] No backups found for {lookupEntity}");
                            Console.WriteLine("[DUMP] All available backup tables:");
                            foreach (var kv in originalMethodTable)
                            {
                                Console.WriteLine($"  [{kv.Key}] contains: {string.Join(", ", kv.Value.Keys)}");
                            }
                            throw new Exception($"No backups found for '{lookupEntity}'");
                        }

                        if (!backups.TryGetValue(methodName, out var original))
                        {
                            Console.WriteLine($"[FATAL] Method '{methodName}' not found in backups of {lookupEntity}");
                            throw new Exception($"Ancestor method '{methodName}' not found in original definitions up the chain.");
                        }

                        Console.WriteLine($"[CALL] Executing ancestor method '{methodName}' from entity '{lookupEntity}'");
                        var ancestorScope = new Interpreter(this.variables)
                        {
                            entityTable = this.entityTable,
                            originalMethodTable = this.originalMethodTable,
                            currentEntity = this.currentEntity,
                            currentInstance = this.currentInstance
                        };

                        for (int i = 0; i < original.Parameters.Count; i++)
                        {
                            ancestorScope.variables[original.Parameters[i]] = (null, false); // 👈 you can support real args later
                        }

                        Console.WriteLine($"[CALL] Executing ancestor method '{methodName}' from entity '{lookupEntity}'");
                        return await ancestorScope.Evaluate(original.Body);

                    }





                case MemberAccessNode memberAccess:
                    {
                        var targetObj = await Evaluate(memberAccess.Target);
                        if (targetObj is Dictionary<string, object> objDict)
                        {
                            if (!objDict.TryGetValue(memberAccess.Member, out var value))
                                throw new Exception($"Property or method '{memberAccess.Member}' not found");

                            return value;
                        }

                        throw new Exception("Cannot access member of non-object");
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

    // String concatenation
if (bin.Operator == "+" && (left is string || right is string))
{
    Console.WriteLine(left?.ToString() + right?.ToString());
    return double.NaN;
}

// Allow any object to be compared with '==' / '!='
if (bin.Operator is "==" or "!=")
{
    bool result = bin.Operator == "==" ? Equals(left, right) : !Equals(left, right);
    return result ? 1 : 0;
}

// Disallow non-numeric math
if (!(left is IConvertible) || !(right is IConvertible))
{
    throw new Exception($"Cannot apply '{bin.Operator}' to non-numeric values: {left?.GetType()} and {right?.GetType()}");
}

// Safe numeric conversion
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





private Dictionary<string, MethodNode> GetAllMethods(EntityDefinition def)
{
    var methods = new Dictionary<string, MethodNode>();

    if (def.Parent != null && entityTable.TryGetValue(def.Parent, out var parentDef) && !def.Disowns)
    {
        foreach (var kv in GetAllMethods(parentDef))
        {
            methods[kv.Key] = kv.Value;
        }
    }

    foreach (var kv in def.Methods)
    {
        methods[kv.Key] = kv.Value;
    }

    return methods;
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

