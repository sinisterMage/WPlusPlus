using System;
using System.Collections.Generic;
using WPlusPlus.AST;
using WPlusPlus.JIT;

namespace WPlusPlus.JIT
{
    

    

    public class BytecodeEmitter
    {
        private List<Instruction> instructions = new();

        public List<Instruction> Emit(Node node)
        {
            EmitNode(node);
            return instructions;
        }
        private int EmitPlaceholder()
{
    instructions.Add(new Instruction(OpCode.Jump, -1)); // temporary -1 offset
    return instructions.Count - 1;
}

private void PatchJump(int index, int target)
{
    instructions[index] = new Instruction(instructions[index].OpCode, target);
}


        private void EmitNode(Node node)
        {
            switch (node)
            {
                case NumberNode num:
                    instructions.Add(new Instruction(OpCode.LoadConst, double.Parse(num.Value)));
                    break;

                case StringNode str:
                    instructions.Add(new Instruction(OpCode.LoadConst, str.Value));
                    break;

                case PrintNode print:
                    EmitNode(print.Expression);
                    instructions.Add(new Instruction(OpCode.Print));
                    break;
                case BooleanNode b:
                    instructions.Add(new Instruction(OpCode.LoadConst, b.Value ? 1.0 : 0.0));
                    break;

                case NullNode:
                    instructions.Add(new Instruction(OpCode.LoadConst, null));
                    break;
                case AssignmentNode assign:
                    EmitNode(assign.Value);
                    instructions.Add(new Instruction(OpCode.StoreVar, assign.Identifier.Name));
                    break;
                case BlockNode block:
                    foreach (var stmt in block.Statements)
                        EmitNode(stmt);
                    break;
                    case IfElseNode ifelse:
                        {
                            EmitNode(ifelse.Condition);
                            int jumpToElse = instructions.Count;
                            instructions.Add(new Instruction(OpCode.JumpIfFalse, -1)); // placeholder

                            EmitNode(ifelse.IfBody);
                            int jumpToEnd = -1;
                            if (ifelse.ElseBody != null)
                            {
                                jumpToEnd = instructions.Count;
                                instructions.Add(new Instruction(OpCode.Jump, -1)); // placeholder
                            }

                            int elseStart = instructions.Count;
                            PatchJump(jumpToElse, elseStart);

                            if (ifelse.ElseBody != null)
                            {
                                EmitNode(ifelse.ElseBody);
                                PatchJump(jumpToEnd, instructions.Count);
                            }

                            break;
                        }
                        case WhileNode loop:
                        {
                            int loopStart = instructions.Count;
                            EmitNode(loop.Condition);

                            int jumpToEnd = instructions.Count;
                            instructions.Add(new Instruction(OpCode.JumpIfFalse, -1)); // placeholder

                            EmitNode(loop.Body);
                            instructions.Add(new Instruction(OpCode.Jump, loopStart)); // loop back

                            int loopEnd = instructions.Count;
                            PatchJump(jumpToEnd, loopEnd);
                            break;
                        }






                case BinaryExpressionNode bin:
                    EmitNode(bin.Left);
                    EmitNode(bin.Right);

                    instructions.Add(bin.Operator switch
                    {
                        "+" => new Instruction(OpCode.Add),
                        "-" => new Instruction(OpCode.Sub),
                        "*" => new Instruction(OpCode.Mul),
                        "/" => new Instruction(OpCode.Div),
                        _ => throw new Exception("Unsupported operator: " + bin.Operator)
                    });
                    break;

                case VariableDeclarationNode decl:
                    EmitNode(decl.Value);
                    instructions.Add(new Instruction(OpCode.StoreVar, decl.Name));
                    break;

                case IdentifierNode id:
                    instructions.Add(new Instruction(OpCode.LoadVar, id.Name));
                    break;

                case CallNode call:
                    foreach (var arg in call.Arguments)
                        EmitNode(arg);
                    EmitNode(call.Callee);
                    instructions.Add(new Instruction(OpCode.Call, call.Arguments.Count));
                    break;

                case ReturnNode ret:
                    EmitNode(ret.Expression);
                    instructions.Add(new Instruction(OpCode.Return));
                    break;

                default:
                    throw new Exception($"BytecodeEmitter: Unsupported node type {node.GetType().Name}");
            }
        }
    }
}
