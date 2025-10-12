use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use once_cell::sync::Lazy;
use std::ffi::{CStr, c_char};

use crate::runtime::core::register_task; // ✅ use shared async runtime

/// Wrapper for raw function pointer used in W++ runtime.
/// This allows it to implement Send/Sync safely.
#[derive(Clone, Copy)]
pub struct WppFunctionRef(pub *const ());
unsafe impl Send for WppFunctionRef {}
unsafe impl Sync for WppFunctionRef {}

#[derive(Clone)]
pub struct Endpoint {
    pub path: String,
    pub handler: WppFunctionRef,
}

// ✅ Explicit thread-safety guarantees for static usage
unsafe impl Send for Endpoint {}
unsafe impl Sync for Endpoint {}

static ENDPOINTS: Lazy<Arc<RwLock<Vec<Endpoint>>>> =
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

/// Register a W++ endpoint with a path and handler reference
pub fn register_endpoint(path: String, handler: WppFunctionRef) {
    let display_path = path.clone();
    ENDPOINTS.write().unwrap().push(Endpoint { path, handler });
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

/// Handle a single TCP client
/// Handle a single TCP client
async fn handle_client(mut socket: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    let mut buffer = [0u8; 1024];
    let size = socket.read(&mut buffer).await?;

    if size == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..size]);
    let path = parse_path(&request);
    println!("🌍 Request from {} → {}", addr, path);

    // ✅ Copy endpoints into a local Vec before awaiting
    let endpoints_snapshot = {
        let endpoints = ENDPOINTS.read().unwrap();
        endpoints.clone()
    }; // lock released here ✅

    let has_endpoint = endpoints_snapshot.iter().any(|e| e.path == path);

    let response = if has_endpoint {
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello from W++!\n"
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n"
    };

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}


/// Simple HTTP path parser
fn parse_path(request: &str) -> String {
    request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string()
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
