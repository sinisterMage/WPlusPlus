namespace WPlusPlus.AST
{
    public class AwaitNode : Node
    {
        public Node Expression { get; }

        public AwaitNode(Node expression)
        {
            Expression = expression;
        }
    }
}


