using System;
using System.IO;
using System.Text;
using WPlusPlus.AST;

namespace WPlusPlus.JIT
{
    public class WppCGenerator
    {
        private StringBuilder sb = new();
        private int indentLevel = 0;

        private void Indent() => sb.Append(new string(' ', indentLevel * 4));
        private void WriteLine(string line = "") => sb.AppendLine(new string(' ', indentLevel * 4) + line);

        public string Generate(Node root)
        {
            sb.Clear();
            WriteLine("#include <stdio.h>");
            WriteLine();
            WriteLine("int main() {");
            indentLevel++;

            EmitNode(root);

            WriteLine("return 0;");
            indentLevel--;
            WriteLine("}");

            return sb.ToString();
        }

        private void EmitNode(Node node)
        {
                Console.WriteLine($"🧠 Emitting node: {node.GetType().Name}");
            switch (node)
            {
                case BlockNode block:
                    foreach (var stmt in block.Statements)
                        EmitNode(stmt);
                    break;

                case VariableDeclarationNode decl:
                    Indent();
                    sb.Append("double " + decl.Name + " = ");
                    EmitExpression(decl.Value);
                    sb.AppendLine(";");
                    break;

                case AssignmentNode assign:
                    Indent();
                    sb.Append(assign.Identifier.Name + " = ");
                    EmitExpression(assign.Value);
                    sb.AppendLine(";");
                    break;

                case PrintNode print:
                    Indent();
                    sb.Append("printf(\"%f\\n\", ");
                    EmitExpression(print.Expression);
                    sb.AppendLine(");");
                    break;

                default:
                    throw new Exception($"Unsupported statement: {node.GetType().Name}");
            }
        }

        private void EmitExpression(Node node)
        {
            switch (node)
            {
                case NumberNode num:
                    sb.Append(num.Value);
                    break;

                case IdentifierNode id:
                    sb.Append(id.Name);
                    break;

                case BinaryExpressionNode bin:
                    sb.Append("(");
                    EmitExpression(bin.Left);
                    sb.Append(" " + bin.Operator + " ");
                    EmitExpression(bin.Right);
                    sb.Append(")");
                    break;

                default:
                    throw new Exception($"Unsupported expression: {node.GetType().Name}");
            }
        }

        public void WriteToFile(Node node, string filePath)
{
    Console.WriteLine("🛠 Entered WriteToFile...");
    var code = Generate(node);
    Console.WriteLine("🛠 Generated C code:\n" + code);

    try
    {
        File.WriteAllText(filePath, code);
        Console.WriteLine($"✅ C code written to: {filePath}");
    }
    catch (Exception ex)
    {
        Console.WriteLine("❌ Failed to write file: " + ex.Message);
    }
}

    }
}
