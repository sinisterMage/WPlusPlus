using System;

static class Program
{
    static int Lcg(int n)
    {
        int a = 1664525;
        int c = 1013904223;
        int x = 0;
        for (int i = 0; i < n; i++)
        {
            unchecked { x = a * x + c; }
        }
        return x;
    }

    static void Main()
    {
        var env = Environment.GetEnvironmentVariable("N");
        int n = 50_000_000;
        if (!string.IsNullOrEmpty(env) && int.TryParse(env, out var parsed))
            n = parsed;

        Console.WriteLine(Lcg(n));
    }
}

