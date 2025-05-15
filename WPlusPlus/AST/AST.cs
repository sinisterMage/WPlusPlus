using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace WPlusPlus.AST
{
    public abstract class Node { }

    public class NumberNode : Node
    {
        public string Value { get; }

        public NumberNode(string value)
        {
            Value = value;
        }
    }

    public class IdentifierNode : Node
    {
        public string Name { get; }

        public IdentifierNode(string name)
        {
            Name = name;
        }
    }

    public class BinaryExpressionNode : Node
    {
        public Node Left { get; }
        public string Operator { get; }
        public Node Right { get; }

        public BinaryExpressionNode(Node left, string op, Node right)
        {
            Left = left;
            Operator = op;
            Right = right;
        }
    }

    public class AssignmentNode : Node
    {
        public IdentifierNode Identifier { get; }
        public Node Value { get; }

        public AssignmentNode(IdentifierNode identifier, Node value)
        {
            Identifier = identifier;
            Value = value;
        }
    }
    public class BreakNode : Node { }

    public class ContinueNode : Node { }
    public class LambdaNode : Node
    {
        public List<string> Parameters { get; }
        public Node Body { get; }

        public LambdaNode(List<string> parameters, Node body)
        {
            Parameters = parameters;
            Body = body;
        }
    }



    public class CallNode : Node
    {
        public Node Callee { get; }
        public List<Node> Arguments { get; }

        public CallNode(Node callee, List<Node> arguments)
        {
            Callee = callee;
            Arguments = arguments;
        }
    }
    public class BooleanNode : Node
    {
        public bool Value { get; }
        public BooleanNode(bool value) => Value = value;
    }

    public class NullNode : Node { }
    public class UnaryExpressionNode : Node
    {
        public string Operator { get; }
        public Node Operand { get; }

        public UnaryExpressionNode(string op, Node operand)
        {
            Operator = op;
            Operand = operand;
        }
    }
    public class TryCatchNode : Node
    {
        public Node TryBlock { get; }
        public string CatchVariable { get; }
        public Node CatchBlock { get; }

        public TryCatchNode(Node tryBlock, string catchVar, Node catchBlock)
        {
            TryBlock = tryBlock;
            CatchVariable = catchVar;
            CatchBlock = catchBlock;
        }
    }
    public class ThrowNode : Node
    {
        public Node Expression { get; }
        public ThrowNode(Node expression) => Expression = expression;
    }
    public class SwitchNode : Node
    {
        public Node Expression { get; }
        public List<(Node CaseValue, List<Node> Body)> Cases { get; }
        public List<Node> DefaultBody { get; }

        public SwitchNode(Node expr, List<(Node, List<Node>)> cases, List<Node> defaultBody)
        {
            Expression = expr;
            Cases = cases;
            DefaultBody = defaultBody;
        }
    }





}

