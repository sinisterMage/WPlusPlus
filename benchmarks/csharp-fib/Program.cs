using System;

static class Program
{
    static int Fib(int n)
    {
        int a = 0;
        int b = 1;
        for (int i = 0; i < n; i++)
        {
            int t = a + b;
            a = b;
            b = t;
        }
        return a;
    }

    static void Main()
    {
        Console.WriteLine(Fib(45));
    }
}

