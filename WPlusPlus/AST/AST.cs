using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using WPlusPlus.AST;

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
    public class EntityNode : Node
    {
        public string Name { get; }
        public string? Parent { get; }
        public bool DisownsParent { get; }
        public List<Node> Body { get; }

        public EntityNode(string name, string? parent, bool disownsParent, List<Node> body)
        {
            Name = name;
            Parent = parent;
            DisownsParent = disownsParent;
            Body = body;
        }
    }
    public class NewNode : Node
    {
        public string EntityName { get; }

        public NewNode(string entityName)
        {
            EntityName = entityName;
        }
    }
    public class AltersNode : Node
    {
        public string ChildEntity { get; }
        public string TargetAncestor { get; }
        public List<Node> AlteredMethods { get; }

        public AltersNode(string childEntity, string targetAncestor, List<Node> alteredMethods)
        {
            ChildEntity = childEntity;
            TargetAncestor = targetAncestor;
            AlteredMethods = alteredMethods;
        }
    }

    public class MethodNode : Node
    {
        public string Name { get; }
        public List<string> Parameters { get; }
        public Node Body { get; }

        public MethodNode(string name, List<string> parameters, Node body)
        {
            Name = name;
            Parameters = parameters;
            Body = body;
        }
    }
    public class MeNode : Node { }
    public class AncestorCallNode : Node
    {
        public string MethodName { get; }
        public List<Node> Arguments { get; }

        public AncestorCallNode(string methodName, List<Node> arguments)
        {
            MethodName = methodName;
            Arguments = arguments;
        }
    }
    public class MemberAccessNode : Node
    {
        public Node Target { get; }
        public string Member { get; }

        public MemberAccessNode(Node target, string member)
        {
            Target = target;
            Member = member;
        }
    }



}




