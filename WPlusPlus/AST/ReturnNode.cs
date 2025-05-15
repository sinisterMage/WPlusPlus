namespace WPlusPlus.AST
{
    public class ReturnNode : Node
    {
        public Node Expression { get; }

        public ReturnNode(Node expression)
        {
            Expression = expression;
        }
    }
}
