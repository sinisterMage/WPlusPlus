using System;
using System.Reflection;
using System.Reflection.Emit;

namespace WPlusPlus
{
    public class WppIlCompiler
    {
        public void CompileHardcodedSample()
        {
            var asmName = new AssemblyName("WppGenerated");
            var asmBuilder = AssemblyBuilder.DefineDynamicAssembly(asmName, AssemblyBuilderAccess.Run);
            var moduleBuilder = asmBuilder.DefineDynamicModule("MainModule");
            var typeBuilder = moduleBuilder.DefineType("Program", TypeAttributes.Public | TypeAttributes.Class);

            var methodBuilder = typeBuilder.DefineMethod("Main", MethodAttributes.Public | MethodAttributes.Static, typeof(void), Type.EmptyTypes);
            var il = methodBuilder.GetILGenerator();

            // === Hardcoded W++ example: print(5 + 10);
            il.Emit(OpCodes.Ldc_I4, 5);
            il.Emit(OpCodes.Ldc_I4, 10);
            il.Emit(OpCodes.Add);
            il.Emit(OpCodes.Call, typeof(Console).GetMethod("WriteLine", new[] { typeof(int) }));
            il.Emit(OpCodes.Ret);

            var programType = typeBuilder.CreateType();
            programType.GetMethod("Main")?.Invoke(null, null);
        }
    }
}
