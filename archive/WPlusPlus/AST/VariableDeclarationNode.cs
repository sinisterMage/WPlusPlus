using WPlusPlus.AST;

public class VariableDeclarationNode : Node
{
    public string Name { get; }
    public Node Value { get; }
    public bool IsConstant { get; }

    public VariableDeclarationNode(string name, Node value, bool isConst)
    {
        Name = name;
        Value = value;
        IsConstant = isConst;
    }
}
