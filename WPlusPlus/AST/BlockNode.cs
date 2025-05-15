namespace WPlusPlus.AST
{
    public class BlockNode : Node
    {
        public List<Node> Statements { get; }

        public BlockNode(List<Node> statements)
        {
            Statements = statements;
        }
    }
}
