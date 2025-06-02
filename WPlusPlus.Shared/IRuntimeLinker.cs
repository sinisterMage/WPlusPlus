namespace WPlusPlus
{
    public interface IRuntimeLinker
    {
        object Invoke(string typeName, string methodName, object[] args);
    }
}
