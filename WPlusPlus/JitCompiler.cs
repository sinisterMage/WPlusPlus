using System;
using System.Collections.Generic;
using System.Reflection;
using System.Reflection.Emit;
using WPlusPlus.AST;

namespace WPlusPlus
{
    public class ReturnEmittedException : Exception { }

    public class JitCompiler
    {
        private Dictionary<string, LocalBuilder> _locals = new();

        public void Compile(Node ast)
{
    var method = new DynamicMethod("WppMain", typeof(void), Type.EmptyTypes);
    var il = method.GetILGenerator();

    Label returnLabel = il.DefineLabel();
    LocalBuilder returnValue = il.DeclareLocal(typeof(object)); // ✅ declared upfront, non-nullable

    EmitNode(ast, il, returnLabel, null, ref returnValue);

    il.MarkLabel(returnLabel);
    il.Emit(OpCodes.Ldloc, returnValue); // ✅ always loaded safely
    il.Emit(OpCodes.Call, typeof(Console).GetMethod("WriteLine", new[] { typeof(object) })); // print return
    il.Emit(OpCodes.Ret);

    var action = (Action)method.CreateDelegate(typeof(Action));
    action(); // Run!
}


        private void EmitNode(Node node, ILGenerator il, Label? breakLabel, Label? continueLabel, ref LocalBuilder? returnValue)
        {
            switch (node)
            {
                case BlockNode block:
                    foreach (var stmt in block.Statements)
                        EmitNode(stmt, il, breakLabel, continueLabel, ref returnValue); // RIGHT
                    break;


                case PrintNode print:
                    EmitNode(print.Expression, il, breakLabel, continueLabel, ref returnValue);
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
                    EmitNode(assign.Value, il, breakLabel, continueLabel, ref returnValue);
                    if (!_locals.ContainsKey(assign.Identifier.Name))
                        _locals[assign.Identifier.Name] = il.DeclareLocal(typeof(object));
                    il.Emit(OpCodes.Stloc, _locals[assign.Identifier.Name]);
                    break;

                case IdentifierNode id:
                    if (_locals.TryGetValue(id.Name, out var local))
                        il.Emit(OpCodes.Ldloc, local);
                    else
                        throw new Exception($"Undefined variable '{id.Name}'");
                    break;

                case BinaryExpressionNode bin:
                    Console.WriteLine($"[JIT] Compiling binary expression: {bin.Operator}");

                    EmitNode(bin.Left, il, breakLabel, continueLabel, ref returnValue);
                    il.Emit(OpCodes.Unbox_Any, typeof(int));
                    var leftTemp = il.DeclareLocal(typeof(int));
                    il.Emit(OpCodes.Stloc, leftTemp);
                    Console.WriteLine("[JIT] Left operand compiled");

                    EmitNode(bin.Right, il, breakLabel, continueLabel, ref returnValue);
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
                    EmitNode(varDecl.Value, il, breakLabel, continueLabel, ref returnValue);

                    if (!_locals.ContainsKey(varDecl.Name))
                        _locals[varDecl.Name] = il.DeclareLocal(typeof(object));

                    il.Emit(OpCodes.Stloc, _locals[varDecl.Name]); // Store value in local
                    break;


                case IfElseNode ifElse:
                    var elseLabel = il.DefineLabel();
                    var endLabel = il.DefineLabel();

                    EmitNode(ifElse.Condition, il, breakLabel, continueLabel, ref returnValue);
                    il.Emit(OpCodes.Unbox_Any, typeof(int));
                    il.Emit(OpCodes.Ldc_I4_0);
                    il.Emit(OpCodes.Ceq);
                    il.Emit(OpCodes.Brtrue, elseLabel);

                    EmitNode(ifElse.IfBody, il, breakLabel, continueLabel, ref returnValue);
                    il.Emit(OpCodes.Br, endLabel);

                    il.MarkLabel(elseLabel);
                    if (ifElse.ElseBody != null)
    EmitNode(ifElse.ElseBody, il, breakLabel, continueLabel, ref returnValue);

                    il.MarkLabel(endLabel);
                    break;

                case WhileNode whileNode:
                    {
                        Console.WriteLine("[JIT] Emitting while loop");

                        var loopStart = il.DefineLabel();
                        var loopEnd = il.DefineLabel();
                        var loopContinueLabel = il.DefineLabel(); // Renamed!

                        il.MarkLabel(loopStart);

                        EmitNode(whileNode.Condition, il, loopEnd, loopContinueLabel, ref returnValue);
                        il.Emit(OpCodes.Unbox_Any, typeof(int));
                        il.Emit(OpCodes.Ldc_I4_0);
                        il.Emit(OpCodes.Ceq);
                        il.Emit(OpCodes.Brtrue, loopEnd); // Exit if condition is false

                        il.MarkLabel(loopContinueLabel);  // ← continue jumps here
                        EmitNode(whileNode.Body, il, loopEnd, loopContinueLabel, ref returnValue);
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
    EmitNode(forNode.Initializer, il, loopEnd, loopContinueLabel, ref returnValue);

                        il.MarkLabel(loopStart);

                        if (forNode.Condition != null)
                        {
                                EmitNode(forNode.Condition, il, loopEnd, loopContinueLabel, ref returnValue);
                            il.Emit(OpCodes.Unbox_Any, typeof(int));
                            il.Emit(OpCodes.Ldc_I4_0);
                            il.Emit(OpCodes.Ceq);
                            il.Emit(OpCodes.Brtrue, loopEnd); // Exit if condition is false
                        }

                        EmitNode(forNode.Body, il, loopEnd, loopContinueLabel, ref returnValue);

                        il.MarkLabel(loopContinueLabel); // ← continue jumps here
                        if (forNode.Increment != null)
                                EmitNode(forNode.Increment, il, loopEnd, loopContinueLabel, ref returnValue);

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
                       EmitNode(switchNode.Expression, il, null, null, ref returnValue);
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

                             EmitNode(caseExpr, il, null, null, ref returnValue);
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

                                EmitNode(actual, il, switchExitLabel, null, ref returnValue);
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

                                EmitNode(actual, il, switchExitLabel, null, ref returnValue);
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
                    EmitNode(unary.Operand, il, null, null, ref returnValue);
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
                case ReturnNode returnNode:
{
    Console.WriteLine("[JIT] Emitting return");

    EmitNode(returnNode.Expression, il, breakLabel, continueLabel, ref returnValue);
    il.Emit(OpCodes.Stloc, returnValue); // ✅ store in already-declared return local
    il.Emit(OpCodes.Br, breakLabel!.Value); // ✅ jump to return label
    break;
}






            }
        }
    }
}
