using System;
using System.Collections.Generic;
using System.Net;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace WPlusPlus
{
    public class ApiServer
    {
        private readonly Dictionary<(string Path, string Method), Func<HttpListenerRequest, HttpListenerResponse, Task>> _routes
            = new();

        private HttpListener _listener;
        private bool _running = false;

        public void AddRoute(string path, string method, Func<HttpListenerRequest, HttpListenerResponse, Task> handler)
        {
            _routes[(path, method.ToUpper())] = handler;
        }

        /// <summary>
        /// Starts the API server
        /// </summary>
        /// <param name="port">Port number</param>
        /// <param name="publicAccess">
        /// If true, binds to all interfaces (requires netsh on Windows).
        /// If false, binds to localhost only (no admin/netsh needed).
        /// </param>
        public void Start(int port, bool publicAccess = false)
        {
            if (_running) return;
            _running = true;

            _listener = new HttpListener();

            string prefix = publicAccess
                ? $"http://+:{port}/" // + = all interfaces
                : $"http://127.0.0.1:{port}/";

            try
            {
                _listener.Prefixes.Add(prefix);
                _listener.Start();
            }
            catch (HttpListenerException ex) when (ex.ErrorCode == 5) // Access denied
            {
                throw new Exception(
                    $"Access denied when trying to bind to {prefix}. " +
                    "On Windows, run this command to allow it:\n" +
                    $"netsh http add urlacl url={prefix} user=Everyone\n" +
                    "Or run W++ as Administrator."
                );
            }

            Console.WriteLine($"[API] Listening on {prefix}");

            // Async loop to handle incoming requests
            Task.Run(async () =>
            {
                while (_running)
                {
                    try
                    {
                        var context = await _listener.GetContextAsync();
                        _ = Task.Run(async () =>
                        {
                            try
                            {
                                if (_routes.TryGetValue((context.Request.Url.AbsolutePath, context.Request.HttpMethod), out var handler))
                                {
                                    await handler(context.Request, context.Response);
                                }
                                else
                                {
                                    context.Response.StatusCode = 404;
                                    byte[] buf = Encoding.UTF8.GetBytes("Not Found");
                                    await context.Response.OutputStream.WriteAsync(buf, 0, buf.Length);
                                    context.Response.Close();
                                }
                            }
                            catch (Exception ex)
                            {
                                context.Response.StatusCode = 500;
                                byte[] buf = Encoding.UTF8.GetBytes("Server Error: " + ex.Message);
                                await context.Response.OutputStream.WriteAsync(buf, 0, buf.Length);
                                context.Response.Close();
                            }
                        });
                    }
                    catch (HttpListenerException)
                    {
                        // Listener stopped, break loop
                        break;
                    }
                }
            });

            // Handle Ctrl+C to stop server
            Console.CancelKeyPress += (s, e) =>
            {
                e.Cancel = true;
                Stop();
                Environment.Exit(0);
            };

            // Keep W++ alive indefinitely
            Thread.Sleep(Timeout.Infinite);
        }

        public void Stop()
        {
            _running = false;
            try
            {
                _listener?.Stop();
                _listener?.Close();
            }
            catch { }
            Console.WriteLine("[API] Server stopped.");
        }
    }
}
