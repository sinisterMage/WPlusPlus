namespace WPlusPlus.AST
{
    public class AsyncLambdaNode : Node
    {
        public List<string> Parameters { get; }
        public Node Body { get; }

        public AsyncLambdaNode(List<string> parameters, Node body)
        {
            Parameters = parameters;
            Body = body;
        }
    }
}
