// In AST/TypeOfNode.cs
namespace WPlusPlus.AST
{
    public class TypeOfNode : Node
    {
        public string TypeName { get; }

        public TypeOfNode(string typeName)
        {
            TypeName = typeName;
        }
    }
}
