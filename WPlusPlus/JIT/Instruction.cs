namespace WPlusPlus.JIT
{
    public enum OpCode
    {
        LoadConst,
        LoadVar,
        StoreVar,
        Add,
        Sub,
        Mul,
        Div,
        Print,
        Call,
        Return,
        Jump,         // Unconditional
JumpIfFalse,  // Pops condition, jumps if false (0.0)
Label        // Optional for debugging

    }

    public struct Instruction
    {
        public OpCode OpCode { get; }
        public object? Operand { get; }

        public Instruction(OpCode op, object? operand = null)
        {
            OpCode = op;
            Operand = operand;
        }

        public override string ToString() => Operand != null ? $"{OpCode} {Operand}" : $"{OpCode}";
    }
}

