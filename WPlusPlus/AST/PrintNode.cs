namespace WPlusPlus.AST
{
    public class PrintNode : Node
    {
        public Node Expression { get; }

        public PrintNode(Node expression)
        {
            Expression = expression;
        }
    }
}
