defmodule PhoenixBenchmark.Application do
  use Application

  def start(_type, _args) do
    IO.puts("========================================")
    IO.puts("Phoenix Benchmark Server")
    IO.puts("========================================")
    IO.puts("")
    IO.puts("Starting server on http://localhost:4000")
    IO.puts("")
    IO.puts("Endpoints:")
    IO.puts("  GET  /")
    IO.puts("  GET  /api/posts")
    IO.puts("  GET  /api/posts/:id")
    IO.puts("  POST /api/posts")
    IO.puts("")

    children = [
      {Plug.Cowboy, scheme: :http, plug: PhoenixBenchmark.Router, options: [port: 4000]}
    ]

    opts = [strategy: :one_for_one, name: PhoenixBenchmark.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
