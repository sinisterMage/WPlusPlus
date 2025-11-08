use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use once_cell::sync::Lazy;
use std::ffi::{CStr, c_char};
use httparse;
use dashmap::DashMap;

use crate::runtime::core::register_task; // ✅ use shared async runtime

// ✅ Pre-compiled HTTP response headers (Phase 1 v2)
const HTTP_200_KEEPALIVE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\nConnection: keep-alive\r\n\r\n";
const HTTP_200_CLOSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\nConnection: close\r\n\r\n";
const HTTP_404_KEEPALIVE: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: keep-alive\r\n\r\n";
const HTTP_404_CLOSE: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

/// Wrapper for raw function pointer used in W++ runtime.
/// This allows it to implement Send/Sync safely.
#[derive(Clone, Copy)]
pub struct WppFunctionRef(pub *const ());
unsafe impl Send for WppFunctionRef {}
unsafe impl Sync for WppFunctionRef {}

// ✅ Phase 2 Optimization #1: Buffer Pool for response buffers
struct BufferPool {
    pool: Mutex<Vec<Vec<u8>>>,
}

impl BufferPool {
    fn new() -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(128)),
        }
    }

    async fn acquire(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().await;
        pool.pop().unwrap_or_else(|| Vec::with_capacity(512))
    }

    async fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        let mut pool = self.pool.lock().await;
        if pool.len() < 256 {
            pool.push(buf);
        }
    }
}

static BUFFER_POOL: Lazy<BufferPool> = Lazy::new(BufferPool::new);

// ✅ Phase 2 Optimization #2: DashMap for lock-free endpoint routing
static ENDPOINTS: Lazy<DashMap<String, WppFunctionRef>> = Lazy::new(DashMap::new);

/// Register a W++ endpoint with a path and handler reference
pub fn register_endpoint(path: String, handler: WppFunctionRef) {
    let display_path = path.clone();
    ENDPOINTS.insert(path, handler);
    println!("🌿 [runtime] Registered endpoint: {}", display_path);
}

/// Start an async TCP-based HTTP server — runs inside the shared Tokio runtime.
async fn start_server_async(port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to bind TCP port");

    println!("🦄 [runtime] Listening at http://0.0.0.0:{}", port);

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                tokio::spawn(async move {
    // Move ownership of the socket into the async block
    if let Err(e) = handle_client(socket, addr).await {
        eprintln!("⚠️ connection error: {}", e);
    }
});

            }
            Err(e) => eprintln!("⚠️ accept error: {e}"),
        }
    }
}

/// Handle a single TCP client with keep-alive support (Phase 2 optimized)
async fn handle_client(mut socket: TcpStream, _addr: SocketAddr) -> tokio::io::Result<()> {
    // ✅ Phase 2: Use stack-allocated buffer for reading (no heap allocation)
    let mut buffer = [0u8; 8192];

    // ✅ Phase 2: Acquire buffer from pool (reuses pre-allocated buffers)
    let mut response_buf = BUFFER_POOL.acquire().await;

    // Keep-alive loop: handle multiple requests on the same connection
    loop {
        let size = socket.read(&mut buffer).await?;

        if size == 0 {
            break; // Client closed connection
        }

        // ✅ Phase 1 v2: Fast path parsing with httparse
        let request_bytes = &buffer[..size];
        let (path, keep_alive) = parse_http_request(request_bytes);

        // ✅ Phase 2 Optimization #2: Lock-free endpoint lookup with DashMap
        // Convert path bytes to string for lookup (zero-copy when possible)
        let path_str = std::str::from_utf8(path).unwrap_or("/");
        let handler_opt = ENDPOINTS.get(path_str).map(|entry| *entry.value());

        // Build response efficiently
        response_buf.clear();

        if let Some(handler) = handler_opt {
            // ✅ DYNAMIC HANDLER INVOCATION
            let _status_code = invoke_handler(handler);

            // ✅ Use pre-compiled headers
            response_buf.extend_from_slice(
                if keep_alive { HTTP_200_KEEPALIVE } else { HTTP_200_CLOSE }
            );
            response_buf.extend_from_slice(b"OK\n");
        } else {
            // ✅ Use pre-compiled 404 headers
            response_buf.extend_from_slice(
                if keep_alive { HTTP_404_KEEPALIVE } else { HTTP_404_CLOSE }
            );
        }

        // ✅ Phase 2 Optimization #3: Pipelined write (single syscall)
        socket.write_all(&response_buf).await?;

        if !keep_alive {
            break; // Client requested close
        }
    }

    // ✅ Phase 2: Return buffer to pool for reuse
    BUFFER_POOL.release(response_buf).await;

    Ok(())
}

/// Fast HTTP request parser using httparse (SIMD-optimized)
#[inline]
fn parse_http_request(request: &[u8]) -> (&[u8], bool) {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    match req.parse(request) {
        Ok(httparse::Status::Complete(_)) => {
            let path = req.path.unwrap_or("/").as_bytes();

            // ✅ HTTP/1.1 default: persistent connections (keep-alive)
            // Only close if client explicitly sends "Connection: close"
            let keep_alive = !req.headers.iter()
                .any(|h| h.name.eq_ignore_ascii_case("Connection")
                      && h.value.eq_ignore_ascii_case(b"close"));

            (path, keep_alive)
        }
        _ => (b"/", false) // Fallback for incomplete/malformed requests
    }
}

/// Invoke a W++ handler function dynamically
fn invoke_handler(handler: WppFunctionRef) -> i32 {
    // Cast the raw function pointer to the correct signature: fn() -> i32
    type HandlerFn = unsafe extern "C" fn() -> i32;

    unsafe {
        let handler_fn: HandlerFn = std::mem::transmute(handler.0);
        handler_fn()
    }
}

/// === C ABI Bindings ===

#[unsafe(no_mangle)]
pub extern "C" fn wpp_register_endpoint(path_ptr: *const c_char, handler_ptr: *const ()) {
    if path_ptr.is_null() {
        eprintln!("❌ Null path pointer");
        return;
    }

    let path = unsafe { CStr::from_ptr(path_ptr) }.to_string_lossy().to_string();
    register_endpoint(path, WppFunctionRef(handler_ptr));
}

/// ✅ New version integrated with async scheduler
#[unsafe(no_mangle)]
pub extern "C" fn wpp_start_server(port: i32) {
    println!("🚀 [runtime] Queuing HTTP server task on port {}", port);
    register_task(async move {
        start_server_async(port as u16).await;
    });
}
