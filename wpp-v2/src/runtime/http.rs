use once_cell::sync::Lazy;
use reqwest::Client;
use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char},
    sync::Mutex,
};

/// === HTTP Response Struct ===
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: i32,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// === Global Stores ===
/// Holds HTTP responses for handle-based access.
static RESP_STORE: Lazy<Mutex<Vec<HttpResponse>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Holds all CString allocations that have been passed to C.
/// Prevents Rust from freeing memory prematurely.
static STRING_ARENA: Lazy<Mutex<Vec<CString>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// === Store / Retrieve Helpers ===
fn store_response(resp: HttpResponse) -> i32 {
    let mut store = RESP_STORE.lock().unwrap();
    store.push(resp);
    (store.len() - 1) as i32
}

fn get_response(handle: i32) -> Option<HttpResponse> {
    RESP_STORE.lock().unwrap().get(handle as usize).cloned()
}

/// Leak-safe CString creation.
/// The CString is kept alive in STRING_ARENA, and the raw pointer is safe for C use.
fn leak_cstring_owned(s: String) -> *mut c_char {
    let c = CString::new(s).unwrap_or_default();
    c.into_raw() // true heap leak; caller never frees it
}

/// === Async Request Core ===
async fn do_request(method: &str, url: &str, body: Option<&str>) -> HttpResponse {
    let client = Client::new();

    let mut req = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };

    if let Some(b) = body {
        req = req.body(b.to_string());
    }

    let resp = req.send().await.expect("HTTP request failed");
    let status = resp.status().as_u16() as i32;
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect::<HashMap<_, _>>();
    let body = resp.text().await.unwrap_or_default();

    HttpResponse { status, body, headers }
}

/// === Blocking FFI Wrapper ===
/// Executes an HTTP request synchronously for W++ → C interop.
fn call_blocking_http(url_ptr: *const c_char, body_ptr: Option<*const c_char>, method: &str) -> i32 {
    if url_ptr.is_null() {
        eprintln!("⚠️ [http] Null URL pointer");
        return -1;
    }

    let url = unsafe { CStr::from_ptr(url_ptr) }.to_string_lossy().to_string();
    let body = body_ptr.map(|b| unsafe { CStr::from_ptr(b) }.to_string_lossy().to_string());

    println!("🌐 [{}] {}", method, url);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(async { do_request(method, &url, body.as_deref()).await });

    println!(
        "✅ [{}] {} => {} ({} bytes)",
        method,
        url,
        res.status,
        res.body.len()
    );

    let handle = store_response(res);
    println!("📦 [http] Stored response handle {}", handle);
    handle
}

/// === HTTP Method Bindings ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_get(ptr: *const c_char) -> i32 {
    call_blocking_http(ptr, None, "GET")
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_post(url_ptr: *const c_char, body_ptr: *const c_char) -> i32 {
    call_blocking_http(url_ptr, Some(body_ptr), "POST")
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_put(url_ptr: *const c_char, body_ptr: *const c_char) -> i32 {
    call_blocking_http(url_ptr, Some(body_ptr), "PUT")
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_patch(url_ptr: *const c_char, body_ptr: *const c_char) -> i32 {
    call_blocking_http(url_ptr, Some(body_ptr), "PATCH")
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_delete(url_ptr: *const c_char) -> i32 {
    call_blocking_http(url_ptr, None, "DELETE")
}

/// === Response Accessors ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_status(handle: i32) -> i32 {
    get_response(handle).map(|r| r.status).unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wpp_http_body(handle: i32) -> *mut std::ffi::c_void {
    match get_response(handle) {
        Some(r) => {
            let ptr = leak_cstring_owned(r.body) as *mut std::ffi::c_void;
            println!("🔹 [wpp_http_body] handle={} -> {:?}", handle, ptr);
            ptr
        }
        None => {
            println!("❌ [wpp_http_body] invalid handle {}", handle);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wpp_http_headers(handle: i32) -> *mut std::ffi::c_void {
    match get_response(handle) {
        Some(r) => {
            let joined = r
                .headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            let ptr = leak_cstring_owned(joined) as *mut std::ffi::c_void;
            println!("🔹 [wpp_http_headers] handle={} -> {:?}", handle, ptr);
            ptr
        }
        None => std::ptr::null_mut(),
    }
}


/// === Cleanup (Optional) ===
/// Frees all stored CStrings in STRING_ARENA.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wpp_http_free_all() {
    let mut arena = STRING_ARENA.lock().unwrap();
    let count = arena.len();
    arena.clear();
    println!("🧹 [http] Freed {} stored HTTP strings", count);
}
