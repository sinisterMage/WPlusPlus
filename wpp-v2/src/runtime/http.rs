use reqwest::Client;
use std::collections::HashMap;
use std::ffi::{CStr, c_char};

/// A basic HTTP response structure for W++ use.
#[derive(Debug)]
pub struct HttpResponse {
    pub status: i32,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// === Internal async handler ===
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

    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16() as i32;
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect::<HashMap<_, _>>();
    let body = resp.text().await.unwrap_or_default();

    HttpResponse { status, body, headers }
}

/// === Async Rust functions for each method (renamed with `_impl_`) ===
pub async fn _impl_http_get(url: &str) -> HttpResponse { do_request("GET", url, None).await }
pub async fn _impl_http_post(url: &str, body: &str) -> HttpResponse { do_request("POST", url, Some(body)).await }
pub async fn _impl_http_put(url: &str, body: &str) -> HttpResponse { do_request("PUT", url, Some(body)).await }
pub async fn _impl_http_patch(url: &str, body: &str) -> HttpResponse { do_request("PATCH", url, Some(body)).await }
pub async fn _impl_http_delete(url: &str) -> HttpResponse { do_request("DELETE", url, None).await }

/// === Synchronous C ABI wrappers ===
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

/// === Shared helper for blocking FFI calls ===
fn call_blocking_http(url_ptr: *const c_char, body_ptr: Option<*const c_char>, method: &str) -> i32 {
    if url_ptr.is_null() {
        eprintln!("⚠️ [http] null pointer passed");
        return -1;
    }

    let url = unsafe { CStr::from_ptr(url_ptr) }.to_string_lossy().to_string();
    let body = body_ptr.map(|b| unsafe { CStr::from_ptr(b) }.to_string_lossy().to_string());

    println!("🌐 [{}] {}", method, url);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(async {
        do_request(method, &url, body.as_deref()).await
    });

    println!("✅ [{}] {} => {} ({} bytes)", method, url, res.status, res.body.len());
    res.status
}
