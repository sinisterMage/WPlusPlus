namespace WPlusPlus.AST
{
    public class ForNode : Node
    {
        public Node Initializer { get; }
        public Node Condition { get; }
        public Node Increment { get; }
        public Node Body { get; }

        public ForNode(Node initializer, Node condition, Node increment, Node body)
        {
            Initializer = initializer;
            Condition = condition;
            Increment = increment;
            Body = body;
        }
    }
}

