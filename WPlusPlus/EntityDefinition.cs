using WPlusPlus.AST;


namespace WPlusPlus
{
    public class EntityDefinition
    {
        public string Name { get; set; }
        public string? Parent { get; set; }
        public bool Disowns { get; set; }
        public Dictionary<string, MethodNode> Methods { get; set; } = new();
    }
}
