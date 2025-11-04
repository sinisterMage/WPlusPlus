using System;

static class Program
{
    static int CountDiv3(int n)
    {
        int c = 0;
        for (int i = 0; i < n; i++)
        {
            int q = i / 3;
            int r = i - q * 3;
            if (r == 0) c++;
        }
        return c;
    }

    static void Main()
    {
        var env = Environment.GetEnvironmentVariable("N");
        int n = 50_000_000;
        if (!string.IsNullOrEmpty(env) && int.TryParse(env, out var parsed))
            n = parsed;

        Console.WriteLine(CountDiv3(n));
    }
}

