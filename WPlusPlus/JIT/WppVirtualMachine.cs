using System;
using System.Collections.Generic;

namespace WPlusPlus.JIT
{
    public class WppVirtualMachine
{
    private readonly List<Instruction> instructions;
    private readonly Stack<object> stack = new();
    private readonly Dictionary<string, object> variables = new();

    private int ip = 0; // Instruction pointer

    public WppVirtualMachine(List<Instruction> instructions)
    {
        this.instructions = instructions;
    }

    public void Run()
    {
        Console.WriteLine("[JIT] Running with WppVirtualMachine");
        while (ip < instructions.Count)
            {
                var instr = instructions[ip++];
                switch (instr.OpCode)
                {
                    case OpCode.LoadConst:
                        stack.Push(instr.Operand);
                        break;

                    case OpCode.Print:
                        Console.WriteLine(stack.Pop());
                        break;

                    case OpCode.Add:
                        {
                            var b = Convert.ToDouble(stack.Pop());
                            var a = Convert.ToDouble(stack.Pop());
                            stack.Push(a + b);
                            break;
                        }

                    case OpCode.Sub:
                        {
                            var b = Convert.ToDouble(stack.Pop());
                            var a = Convert.ToDouble(stack.Pop());
                            stack.Push(a - b);
                            break;
                        }

                    case OpCode.Mul:
                        {
                            var b = Convert.ToDouble(stack.Pop());
                            var a = Convert.ToDouble(stack.Pop());
                            stack.Push(a * b);
                            break;
                        }

                    case OpCode.Div:
                        {
                            var b = Convert.ToDouble(stack.Pop());
                            var a = Convert.ToDouble(stack.Pop());
                            stack.Push(a / b);
                            break;
                        }

                    case OpCode.StoreVar:
                        variables[instr.Operand!.ToString()!] = stack.Pop();
                        break;

                    case OpCode.LoadVar:
                        {
                            var name = instr.Operand!.ToString()!;
                            if (!variables.TryGetValue(name, out var value))
                                throw new Exception($"Undefined variable: {name}");
                            stack.Push(value);
                            break;
                        }
                        case OpCode.Jump:
    ip = (int)instr.Operand;
    continue;

case OpCode.JumpIfFalse:
    var condition = stack.Pop();
    if (Convert.ToDouble(condition) == 0)
    {
        ip = (int)instr.Operand;
        continue;
    }
    break;


                    case OpCode.Return:
                        return;

                    default:
                        throw new Exception($"Unknown opcode: {instr.OpCode}");
                }
            }
    }
}

}
