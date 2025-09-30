using WPlusPlus.AST;

public class IfElseNode : Node
{
    public Node Condition { get; }
    public Node IfBody { get; }
    public Node ElseBody { get; }

    public IfElseNode(Node condition, Node ifBody, Node elseBody = null)
    {
        Condition = condition;
        IfBody = ifBody;
        ElseBody = elseBody;
    }
}

