namespace WPlusPlus.AST
{
    public class StringNode : Node
    {
        public string Value { get; }

        public StringNode(string value)
        {
            Value = value;
        }
    }
}
