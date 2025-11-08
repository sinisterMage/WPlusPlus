// ASP.NET Core HTTP Benchmark
// Minimal API for performance comparison with Raython and Phoenix

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

// Root endpoint
app.MapGet("/", () => "Hello from ASP.NET Core!");

// API endpoints
app.MapGet("/api/posts", () => Results.Json(new
{
    posts = new[]
    {
        new { id = 1, title = "First Post", author = "Alice" },
        new { id = 2, title = "Second Post", author = "Bob" },
        new { id = 3, title = "Third Post", author = "Charlie" }
    }
}));

app.MapGet("/api/posts/{id:int}", (int id) =>
{
    if (id < 1 || id > 3)
        return Results.NotFound();

    return Results.Json(new
    {
        id,
        title = $"Post {id}",
        author = "Author",
        content = "Sample content for benchmark testing"
    });
});

app.MapPost("/api/posts", () => Results.Json(new
{
    id = 4,
    title = "New Post",
    status = "created"
}));

Console.WriteLine("========================================");
Console.WriteLine("ASP.NET Core Benchmark Server");
Console.WriteLine("========================================");
Console.WriteLine();
Console.WriteLine("Starting server on http://localhost:5000");
Console.WriteLine();
Console.WriteLine("Endpoints:");
Console.WriteLine("  GET  /");
Console.WriteLine("  GET  /api/posts");
Console.WriteLine("  GET  /api/posts/:id");
Console.WriteLine("  POST /api/posts");
Console.WriteLine();

app.Run("http://0.0.0.0:5000");
