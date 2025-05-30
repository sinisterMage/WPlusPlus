using System;
using System.Collections.Generic;
using System.Reflection;
using System.Reflection.Emit;
using WPlusPlus.AST;
using System.Threading.Tasks;

namespace WPlusPlus
{
    public class ReturnEmittedException : Exception { }

    public class JitCompiler
    {

        public class FunctionObject
        {
            private readonly Func<List<object>, Task<object>> _func;

            public FunctionObject(Func<List<object>, Task<object>> func)
            {
                _func = func;
            }

            public Task<object> Invoke(List<object> args)
            {
                return _func(args);
            }
        }
public static Task<object> AwaitTask(Task<object> task)
{
    if (task == null)
        throw new Exception("Null Task in Await");

    return task; // <- return the task itself, not its result
}




        
        private static readonly List<object> _constants = new();


       public async Task Compile(Node ast)

{
    var method = new DynamicMethod("WppMain", typeof(Task<object>), Type.EmptyTypes);
var il = method.GetILGenerator();

Label returnLabel = il.DefineLabel();

// Step 1: Declare raw object local
LocalBuilder returnValue = il.DeclareLocal(typeof(Task<object>));
il.Emit(OpCodes.Ldnull); // 🧠 Push null (of type object)
il.Emit(OpCodes.Call, typeof(Task).GetMethod("FromResult")!.MakeGenericMethod(typeof(object)));
il.Emit(OpCodes.Stloc, returnValue);


var scopedLocals = new Dictionary<string, LocalBuilder>();

try
{
    EmitNode(ast, il, scopedLocals, returnLabel, null, ref returnValue,
             isLambda: false, isAsyncLambda: true, lambdaReturnLabel: returnLabel);
}
catch (Exception ex)
{
    Console.WriteLine("[JIT Compile Error] " + ex.Message);
    Console.WriteLine(ex.StackTrace);
    return;
}

// Step 2: Mark return label
il.MarkLabel(returnLabel);

// Step 3: Load returnValue (object) and wrap it in Task.FromResult<object>
// No need to wrap with FromResult — it's already Task<object>
il.Emit(OpCodes.Ldloc, returnValue);
il.Emit(OpCodes.Ret);
          // ✅ Return Task<object>


    Console.WriteLine("🚀 Running JIT compiled W++ code...");
    var func = (Func<Task<object>>)method.CreateDelegate(typeof(Func<Task<object>>));
var result = await func();
Console.WriteLine($"✅ Result: {result}");

}






        private void EmitNode(
    Node node,
    ILGenerator il,
    Dictionary<string, LocalBuilder> locals,
    Label? breakLabel,
    Label? continueLabel,
    ref LocalBuilder? returnValue,
    bool isLambda = false,
    bool isAsyncLambda = false,
    Label? lambdaReturnLabel = null)

        {
            switch (node)
            {
                case BlockNode block:
                    foreach (var stmt in block.Statements)
                        EmitNode(stmt, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    break;


                case PrintNode print:
                    EmitNode(print.Expression, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    il.Emit(OpCodes.Call, typeof(Console).GetMethod("WriteLine", new[] { typeof(object) }));
                    break;

                case NumberNode num:
                    il.Emit(OpCodes.Ldc_I4, int.Parse(num.Value));
                    il.Emit(OpCodes.Box, typeof(int));
                    break;

                case StringNode str:
                    il.Emit(OpCodes.Ldstr, str.Value);
                    break;

                case AssignmentNode assign:
                    EmitNode(assign.Value, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    if (!locals.ContainsKey(assign.Identifier.Name))
    locals[assign.Identifier.Name] = il.DeclareLocal(typeof(object));
il.Emit(OpCodes.Stloc, locals[assign.Identifier.Name]);

                    break;

                case IdentifierNode id:
                    if (locals.TryGetValue(id.Name, out var local))
                        il.Emit(OpCodes.Ldloc, local);
                    else
                        throw new Exception($"Undefined variable '{id.Name}'");
                    break;

                case BinaryExpressionNode bin:
                    Console.WriteLine($"[JIT] Compiling binary expression: {bin.Operator}");

                    EmitNode(bin.Left, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    il.Emit(OpCodes.Unbox_Any, typeof(int));
                    var leftTemp = il.DeclareLocal(typeof(int));
                    il.Emit(OpCodes.Stloc, leftTemp);
                    Console.WriteLine("[JIT] Left operand compiled");

                    EmitNode(bin.Right, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    il.Emit(OpCodes.Unbox_Any, typeof(int));
                    var rightTemp = il.DeclareLocal(typeof(int));
                    il.Emit(OpCodes.Stloc, rightTemp);
                    Console.WriteLine("[JIT] Right operand compiled");

                    il.Emit(OpCodes.Ldloc, leftTemp);
                    il.Emit(OpCodes.Ldloc, rightTemp);

                    switch (bin.Operator)
                    {
                        case "+": il.Emit(OpCodes.Add); break;
                        case "-": il.Emit(OpCodes.Sub); break;
                        case "*": il.Emit(OpCodes.Mul); break;
                        case "/": il.Emit(OpCodes.Div); break;
                        case "<": il.Emit(OpCodes.Clt); break;
                        case ">": il.Emit(OpCodes.Cgt); break;
                        case "==": il.Emit(OpCodes.Ceq); break;
                        case "!=":
                            il.Emit(OpCodes.Ceq);
                            il.Emit(OpCodes.Ldc_I4_0);
                            il.Emit(OpCodes.Ceq);
                            break;
                        default:
                            throw new Exception($"[JIT ERROR] Unsupported binary operator '{bin.Operator}'");
                    }

                    Console.WriteLine($"[JIT] Operator '{bin.Operator}' emitted successfully");

                    il.Emit(OpCodes.Box, typeof(int));
                    break;



                case VariableDeclarationNode varDecl:
                    EmitNode(varDecl.Value, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                    if (!locals.ContainsKey(varDecl.Name))
    locals[varDecl.Name] = il.DeclareLocal(typeof(object));

il.Emit(OpCodes.Stloc, locals[varDecl.Name]);


                   
                    break;


                case IfElseNode ifElse:
                    var elseLabel = il.DefineLabel();
                    var endLabel = il.DefineLabel();

                    EmitNode(ifElse.Condition, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    il.Emit(OpCodes.Unbox_Any, typeof(int));
                    il.Emit(OpCodes.Ldc_I4_0);
                    il.Emit(OpCodes.Ceq);
                    il.Emit(OpCodes.Brtrue, elseLabel);

                  EmitNode(ifElse.IfBody, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    il.Emit(OpCodes.Br, endLabel);

                    il.MarkLabel(elseLabel);
                    if (ifElse.ElseBody != null)
    EmitNode(ifElse.ElseBody, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                    il.MarkLabel(endLabel);
                    break;

                case WhileNode whileNode:
                    {
                        Console.WriteLine("[JIT] Emitting while loop");

                        var loopStart = il.DefineLabel();
                        var loopEnd = il.DefineLabel();
                        var loopContinueLabel = il.DefineLabel(); // Renamed!

                        il.MarkLabel(loopStart);

                        EmitNode(whileNode.Condition, il, locals, loopEnd, loopContinueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                        il.Emit(OpCodes.Unbox_Any, typeof(int));
                        il.Emit(OpCodes.Ldc_I4_0);
                        il.Emit(OpCodes.Ceq);
                        il.Emit(OpCodes.Brtrue, loopEnd); // Exit if condition is false

                        il.MarkLabel(loopContinueLabel);  // ← continue jumps here
                        EmitNode(whileNode.Body, il, locals, loopEnd, loopContinueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                        il.Emit(OpCodes.Br, loopStart);

                        il.MarkLabel(loopEnd);
                        Console.WriteLine("[JIT] Finished emitting while loop");
                        break;
                    }



                case ForNode forNode:
                    {
                        Console.WriteLine("[JIT] Emitting for loop");

                        var loopStart = il.DefineLabel();
                        var loopEnd = il.DefineLabel();
                        var loopContinueLabel = il.DefineLabel(); // Renamed!

                        if (forNode.Initializer != null)
    EmitNode(forNode.Initializer, il, locals, loopEnd, loopContinueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                        il.MarkLabel(loopStart);

                        if (forNode.Condition != null)
                        {
                            EmitNode(forNode.Condition, il, locals, loopEnd, loopContinueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                            il.Emit(OpCodes.Unbox_Any, typeof(int));
                            il.Emit(OpCodes.Ldc_I4_0);
                            il.Emit(OpCodes.Ceq);
                            il.Emit(OpCodes.Brtrue, loopEnd); // Exit if condition is false
                        }

                        EmitNode(forNode.Body, il, locals, loopEnd, loopContinueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                        il.MarkLabel(loopContinueLabel); // ← continue jumps here
                        if (forNode.Increment != null)
                                EmitNode(forNode.Increment, il, locals, loopEnd, loopContinueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                        il.Emit(OpCodes.Br, loopStart);

                        il.MarkLabel(loopEnd);
                        Console.WriteLine("[JIT] Finished emitting for loop");
                        break;
                    }




                case BreakNode:
                    if (breakLabel == null)
                        throw new Exception("break used outside of a loop");
                    il.Emit(OpCodes.Br, breakLabel.Value);
                    break;

                case ContinueNode:
                    if (continueLabel == null)
                        throw new Exception("continue used outside of a loop");
                    il.Emit(OpCodes.Br, continueLabel.Value);
                    break;
                case SwitchNode switchNode:
                    {
                        Console.WriteLine("[JIT] Emitting switch statement");

                        // Evaluate switch expression and store it
                       EmitNode(switchNode.Expression, il, locals, null, null, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                        il.Emit(OpCodes.Unbox_Any, typeof(int));
                        var switchVal = il.DeclareLocal(typeof(int));
                        il.Emit(OpCodes.Stloc, switchVal);
                        Console.WriteLine($"[JIT] Switch expression stored in local {switchVal.LocalIndex}");

                        // Prepare labels
                        var switchExitLabel = il.DefineLabel();
                        var caseLabels = new List<Label>();
                        var afterCaseLabels = new List<Label>(); // ensure valid jumps
                        var defaultLabel = switchNode.DefaultBody != null ? il.DefineLabel() : switchExitLabel;

                        // Emit comparisons and jumps
                        for (int i = 0; i < switchNode.Cases.Count; i++)
                        {
                            var (caseExpr, _) = switchNode.Cases[i];
                            var caseLabel = il.DefineLabel();
                            var afterCase = il.DefineLabel();

                            caseLabels.Add(caseLabel);
                            afterCaseLabels.Add(afterCase);

                            Console.WriteLine($"[JIT] Emitting comparison for case {i}");

                            EmitNode(caseExpr, il, locals, null, null, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                            il.Emit(OpCodes.Unbox_Any, typeof(int));
                            var caseVal = il.DeclareLocal(typeof(int));
                            il.Emit(OpCodes.Stloc, caseVal);

                            il.Emit(OpCodes.Ldloc, switchVal);
                            il.Emit(OpCodes.Ldloc, caseVal);
                            il.Emit(OpCodes.Ceq);
                            il.Emit(OpCodes.Brtrue, caseLabel);
                        }

                        // No match → go to default or exit
                        if (switchNode.DefaultBody != null)
                        {
                            Console.WriteLine("[JIT] No case matched, jumping to default");
                            il.Emit(OpCodes.Br, defaultLabel);
                        }
                        else
                        {
                            Console.WriteLine("[JIT] No case matched, exiting switch");
                            il.Emit(OpCodes.Br, switchExitLabel);
                        }

                        // Emit case bodies
                        for (int i = 0; i < switchNode.Cases.Count; i++)
                        {
                            var caseLabel = caseLabels[i];
                            var afterCase = afterCaseLabels[i];

                            il.MarkLabel(caseLabel);
                            il.Emit(OpCodes.Nop);

                            Console.WriteLine($"[JIT] Emitting case {i}");

                            foreach (var stmt in switchNode.Cases[i].Item2)
                            {
                                Node actual = stmt is StringNode or NumberNode or IdentifierNode
                                    ? new PrintNode((dynamic)stmt)
                                    : stmt;

                                EmitNode(actual, il, locals, switchExitLabel, null, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                            }

                            il.Emit(OpCodes.Br, switchExitLabel);
                            il.MarkLabel(afterCase);
                        }

                        // Emit default body
                        if (switchNode.DefaultBody != null)
                        {
                            il.MarkLabel(defaultLabel);
                            il.Emit(OpCodes.Nop);
                            Console.WriteLine("[JIT] Emitting default case");

                            foreach (var stmt in switchNode.DefaultBody)
                            {
                                Node actual = stmt is StringNode or NumberNode or IdentifierNode
                                    ? new PrintNode((dynamic)stmt)
                                    : stmt;

                                EmitNode(actual, il, locals, switchExitLabel, null, ref returnValue); // in default body
                            }
                        }

                        il.MarkLabel(switchExitLabel);
                        il.Emit(OpCodes.Nop);
                        Console.WriteLine("[JIT] Finished emitting switch");
                        break;
                    }
                case BooleanNode boolNode:
                    il.Emit(boolNode.Value ? OpCodes.Ldc_I4_1 : OpCodes.Ldc_I4_0);
                    il.Emit(OpCodes.Box, typeof(int)); // ✅ match boxed int type
                    break;
                case UnaryExpressionNode unary:
                    EmitNode(unary.Operand, il, locals, null, null, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                    il.Emit(OpCodes.Unbox_Any, typeof(int));

                    switch (unary.Operator)
                    {
                        case "!":
                            il.Emit(OpCodes.Ldc_I4_0);
                            il.Emit(OpCodes.Ceq); // flip 0 ↔ 1
                            break;
                        default:
                            throw new Exception($"[JIT] Unsupported unary operator '{unary.Operator}'");
                    }

                    il.Emit(OpCodes.Box, typeof(int));
                    break;
                case ReturnNode retNode:
{
    Console.WriteLine("[DEBUG] Handling ReturnNode");
    Console.WriteLine($"[DEBUG] isLambda: {isLambda}, lambdaReturnLabel: {lambdaReturnLabel?.GetHashCode().ToString() ?? "null"}");

    if (retNode.Expression != null)
    {
        Console.WriteLine("[DEBUG] Emitting ReturnNode expression");
        EmitNode(retNode.Expression, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

        if (returnValue == null)
            returnValue = il.DeclareLocal(typeof(object));

        il.Emit(OpCodes.Stloc, returnValue);
    }

    if (lambdaReturnLabel == null)
        throw new InvalidOperationException("Return statement without a return label");

    Console.WriteLine("[DEBUG] Branching to return label");
    il.Emit(OpCodes.Br, lambdaReturnLabel.Value);

    break;
}







                case ThrowNode throwNode:
                    {
                        Console.WriteLine("[JIT] Emitting throw");

                        // Evaluate the expression (e.g., string or variable containing the error message)
                        EmitNode(throwNode.Expression, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                        // Cast the object to string (if it isn't already)
                        il.Emit(OpCodes.Castclass, typeof(string));

                        // Create new Exception(string)
                        var ctor = typeof(Exception).GetConstructor(new[] { typeof(string) });
                        il.Emit(OpCodes.Newobj, ctor);

                        // Throw it
                        il.Emit(OpCodes.Throw);
                        break;
                    }
                case TryCatchNode tryCatch:
                    {
                        Console.WriteLine("[JIT] Emitting try/catch with variable binding");

                        var exLocal = il.DeclareLocal(typeof(Exception));
                        LocalBuilder? stringMessage = null;

                        // Begin try block
                        il.BeginExceptionBlock();
                        EmitNode(tryCatch.TryBlock, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
                        // Begin catch block
                        il.BeginCatchBlock(typeof(Exception));
                        il.Emit(OpCodes.Stloc, exLocal);

                        // Extract message as string and bind to catch variable
                        il.Emit(OpCodes.Ldloc, exLocal); // load Exception
                        var getMessage = typeof(Exception).GetProperty("Message")?.GetGetMethod();
                        il.Emit(OpCodes.Callvirt, getMessage); // call get_Message

                        // Store into variable accessible in catch block
                        if (!string.IsNullOrEmpty(tryCatch.CatchVariable))
                        {
                            if (!locals.ContainsKey(tryCatch.CatchVariable))
{
    stringMessage = il.DeclareLocal(typeof(object));
    locals[tryCatch.CatchVariable] = stringMessage;
}
else
{
    stringMessage = locals[tryCatch.CatchVariable];
}


                            il.Emit(OpCodes.Stloc, stringMessage);
                        }
                        else
                        {
                            // If no variable name, just pop message off stack
                            il.Emit(OpCodes.Pop);
                        }

                        // Emit catch block
                       EmitNode(tryCatch.CatchBlock, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);

                        il.EndExceptionBlock();
                        break;
                    }
                case LambdaNode lambda:
{
    Console.WriteLine("[JIT] Emitting LambdaNode");

    var lambdaParams = lambda.Parameters.ToArray();

    var lambdaMethod = new DynamicMethod(
        "lambda_" + Guid.NewGuid().ToString("N"),
        typeof(Task<object>),
        new Type[] { typeof(List<object>) },
        typeof(JitCompiler).Module,
        true
    );

    var lambdaIL = lambdaMethod.GetILGenerator();
    var localMap = new Dictionary<string, LocalBuilder>();

    // Declare parameters as local variables
    for (int i = 0; i < lambdaParams.Length; i++)
    {
        var paramLocal = lambdaIL.DeclareLocal(typeof(object));
        lambdaIL.Emit(OpCodes.Ldarg_0); // Load List<object>
        lambdaIL.Emit(OpCodes.Ldc_I4, i); // Load index
        lambdaIL.Emit(OpCodes.Callvirt, typeof(List<object>).GetMethod("get_Item")!);
        lambdaIL.Emit(OpCodes.Stloc, paramLocal);
        localMap[lambdaParams[i]] = paramLocal;

        Console.WriteLine($"[JIT] Parameter '{lambdaParams[i]}' mapped to local");
    }

    // Prepare return value and label
    var lambdaReturnValue = lambdaIL.DeclareLocal(typeof(object));
lambdaIL.Emit(OpCodes.Ldnull);                  // ✅ Always init to null
lambdaIL.Emit(OpCodes.Stloc, lambdaReturnValue);

    var returnLabel = lambdaIL.DefineLabel();

    // Emit body
    EmitNode(lambda.Body, lambdaIL, localMap,
        breakLabel: null,
        continueLabel: null,
        ref lambdaReturnValue,
        isLambda: true,
        isAsyncLambda: false,
        lambdaReturnLabel: returnLabel);

    // Ensure return path
    lambdaIL.MarkLabel(returnLabel);
    lambdaIL.Emit(OpCodes.Ldloc, lambdaReturnValue);
    lambdaIL.Emit(OpCodes.Call, typeof(Task).GetMethod("FromResult")!.MakeGenericMethod(typeof(object)));
    lambdaIL.Emit(OpCodes.Ret);

    // Wrap and store in constants
    var del = (Func<List<object>, Task<object>>)lambdaMethod.CreateDelegate(typeof(Func<List<object>, Task<object>>));
    var func = new FunctionObject(del);
    _constants.Add(func);
    int constIndex = _constants.Count - 1;

    // Load FunctionObject from constants list
    il.Emit(OpCodes.Ldsfld, typeof(JitCompiler).GetField(nameof(_constants), BindingFlags.NonPublic | BindingFlags.Static)!);
    il.Emit(OpCodes.Ldc_I4, constIndex);
    il.Emit(OpCodes.Callvirt, typeof(List<object>).GetMethod("get_Item")!);
    il.Emit(OpCodes.Castclass, typeof(FunctionObject));

    Console.WriteLine("[JIT] LambdaNode emission complete");
    break;
}

            case CallNode call:
{
    Console.WriteLine("[JIT] Emitting CallNode");

    // Emit the callee (should result in a FunctionObject on the stack)
    EmitNode(call.Callee, il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
    il.Emit(OpCodes.Castclass, typeof(FunctionObject));
    
    // Store callee into a local
    var funcLocal = il.DeclareLocal(typeof(FunctionObject));
    il.Emit(OpCodes.Stloc, funcLocal);
    Console.WriteLine("[JIT] Stored function in local variable");

    // Create and store argument list
    var argsLocal = il.DeclareLocal(typeof(List<object>));
    var listCtor = typeof(List<object>).GetConstructor(Type.EmptyTypes)
                  ?? throw new InvalidOperationException("Missing List<object> constructor");
    il.Emit(OpCodes.Newobj, listCtor);
    il.Emit(OpCodes.Stloc, argsLocal);

    // Emit all arguments and add to list
    for (int i = 0; i < call.Arguments.Count; i++)
    {
        il.Emit(OpCodes.Ldloc, argsLocal);
        EmitNode(call.Arguments[i], il, locals, breakLabel, continueLabel, ref returnValue, isLambda, isAsyncLambda, lambdaReturnLabel);
        var addMethod = typeof(List<object>).GetMethod("Add")
                       ?? throw new InvalidOperationException("Missing List<object>.Add method");
        il.Emit(OpCodes.Callvirt, addMethod);
    }
    Console.WriteLine("[JIT] All arguments added to list");

    // Call FunctionObject.Invoke(List<object>)
    il.Emit(OpCodes.Ldloc, funcLocal);
    il.Emit(OpCodes.Ldloc, argsLocal);

    var invokeMethod = typeof(FunctionObject).GetMethod("Invoke")
                       ?? throw new InvalidOperationException("FunctionObject.Invoke method not found");
    il.Emit(OpCodes.Callvirt, invokeMethod); // returns Task<object>
    Console.WriteLine("[JIT] Called FunctionObject.Invoke");

    // Replace direct .Result with safe helper method call
    var awaitHelper = typeof(JitCompiler).GetMethod(nameof(AwaitTask), BindingFlags.Public | BindingFlags.Static)
                     ?? throw new InvalidOperationException("AwaitTask method not found");
    il.Emit(OpCodes.Call, awaitHelper);
    Console.WriteLine("[JIT] Awaited using AwaitTask");

    // Declare returnValue if needed
    if (returnValue == null)
        returnValue = il.DeclareLocal(typeof(object));

    // Store in returnValue
    il.Emit(OpCodes.Stloc, returnValue);
    Console.WriteLine("[JIT] Stored result into returnValue");

    break;
}
















            }
        }
        public static object GetConstant(int index)
{
    return _constants[index];
}

    }
}
