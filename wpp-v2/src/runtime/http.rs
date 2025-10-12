use reqwest::Client;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

/// A basic HTTP response structure for W++ use.
#[derive(Debug)]
pub struct HttpResponse {
    pub status: i32,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// Perform HTTP GET request asynchronously.
pub async fn wpp_http_get_async(url: &str) -> HttpResponse {
    let client = Client::new();
    let resp = client.get(url).send().await.unwrap();

    let status = resp.status().as_u16() as i32;
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect::<HashMap<_, _>>();
    let body = resp.text().await.unwrap_or_default();

    HttpResponse { status, body, headers }
}

/// Test helper — synchronous wrapper for quick debugging
pub fn test_http_get(url: &str) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(wpp_http_get_async(url));
    println!(
        "🌐 GET {} => {} ({} bytes)",
        url,
        res.status,
        res.body.len()
    );
}
#[unsafe(no_mangle)]
pub extern "C" fn wpp_http_get(url_ptr: *const c_char) -> i32 {
    if url_ptr.is_null() {
        eprintln!("⚠️ [http] null pointer passed to wpp_http_get");
        return -1;
    }

    // Convert C string → Rust String
    let url = unsafe { CStr::from_ptr(url_ptr) }.to_string_lossy().to_string();
    println!("🌐 [http] GET {}", url);

    // Create a short-lived async runtime for this call
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("❌ [http] failed to create runtime: {e}");
            return -1;
        }
    };

    // Execute the async GET and wait synchronously for completion
    let res = rt.block_on(async { crate::runtime::http::wpp_http_get_async(&url).await });

    println!(
        "✅ [http] {} => status {}, {} bytes",
        url,
        res.status,
        res.body.len()
    );

    // For now, just return status code (integer)
    res.status
}