namespace WPlusPlus.AST
{
    public class ImportNode : Node
    {
        public string Path { get; }

        public ImportNode(string path)
        {
            Path = path;
        }
    }
}
