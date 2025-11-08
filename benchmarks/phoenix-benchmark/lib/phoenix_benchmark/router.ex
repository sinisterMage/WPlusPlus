defmodule PhoenixBenchmark.Router do
  use Plug.Router

  plug :match
  plug Plug.Parsers, parsers: [:json], json_decoder: Jason
  plug :dispatch

  # Root endpoint
  get "/" do
    send_resp(conn, 200, "Hello from Phoenix!")
  end

  # GET /api/posts - List all posts
  get "/api/posts" do
    posts = [
      %{id: 1, title: "First Post", author: "Alice"},
      %{id: 2, title: "Second Post", author: "Bob"},
      %{id: 3, title: "Third Post", author: "Charlie"}
    ]

    conn
    |> put_resp_content_type("application/json")
    |> send_resp(200, Jason.encode!(%{posts: posts}))
  end

  # GET /api/posts/:id - Show single post
  get "/api/posts/:id" do
    id = String.to_integer(id)

    if id >= 1 and id <= 3 do
      post = %{
        id: id,
        title: "Post #{id}",
        author: "Author",
        content: "Sample content for benchmark testing"
      }

      conn
      |> put_resp_content_type("application/json")
      |> send_resp(200, Jason.encode!(post))
    else
      send_resp(conn, 404, "Not Found")
    end
  end

  # POST /api/posts - Create new post
  post "/api/posts" do
    response = %{
      id: 4,
      title: "New Post",
      status: "created"
    }

    conn
    |> put_resp_content_type("application/json")
    |> send_resp(201, Jason.encode!(response))
  end

  # Catch-all for unmatched routes
  match _ do
    send_resp(conn, 404, "Not Found")
  end
end
