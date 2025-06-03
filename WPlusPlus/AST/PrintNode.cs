namespace WPlusPlus.AST
{
    public class PrintNode : Node
{
    public List<Node> Arguments { get; }

    public PrintNode(List<Node> arguments)
    {
        Arguments = arguments;
    }
}

}
