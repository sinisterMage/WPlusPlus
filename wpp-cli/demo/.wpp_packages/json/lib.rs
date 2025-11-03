use libc::{c_char, c_int};
use serde_json::{Value, from_str, to_string};
use std::ffi::{CStr, CString};
use std::ptr;

// Helper: Convert C string to Rust String
unsafe fn c_str_to_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        return None;
    }
    CStr::from_ptr(c_str)
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

// Helper: Convert Rust String to C string (caller must free)
fn string_to_c_str(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Parse JSON string and return as W++ object pointer
/// Returns: Pointer to JSON string (W++ will parse it as object)
///
/// Note: W++ doesn't have native object representation in FFI,
/// so we return the validated JSON string that W++ can parse internally
#[no_mangle]
pub extern "C" fn json_parse(json_str: *const c_char) -> *mut c_char {
    unsafe {
        let input = match c_str_to_string(json_str) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        // Parse to validate
        match from_str::<Value>(&input) {
            Ok(value) => {
                // Return the JSON as string (W++ will handle object conversion)
                string_to_c_str(to_string(&value).unwrap_or_default())
            }
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Convert W++ object to JSON string
///
/// For now, assumes input is already a JSON string (W++ runtime limitation)
/// In future, could accept W++ object pointer and serialize
#[no_mangle]
pub extern "C" fn json_stringify(obj_json: *const c_char) -> *mut c_char {
    unsafe {
        let input = match c_str_to_string(obj_json) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        // Parse and re-stringify to minify
        match from_str::<Value>(&input) {
            Ok(value) => {
                match to_string(&value) {
                    Ok(json) => string_to_c_str(json),
                    Err(_) => ptr::null_mut(),
                }
            }
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Pretty-print JSON with indentation
#[no_mangle]
pub extern "C" fn json_pretty(obj_json: *const c_char, indent: c_int) -> *mut c_char {
    unsafe {
        let input = match c_str_to_string(obj_json) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match from_str::<Value>(&input) {
            Ok(value) => {
                // Create pretty JSON with custom indentation
                let indent_str = " ".repeat(indent as usize);
                let formatted = format_value(&value, &indent_str, 0);
                string_to_c_str(formatted)
            }
            Err(_) => ptr::null_mut(),
        }
    }
}

// Helper for custom pretty printing
fn format_value(value: &Value, indent_str: &str, depth: usize) -> String {
    let current_indent = indent_str.repeat(depth);
    let next_indent = indent_str.repeat(depth + 1);

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }
            let mut result = "{\n".to_string();
            let entries: Vec<_> = map.iter().collect();
            for (i, (key, val)) in entries.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format!("\"{}\": {}", key, format_value(val, indent_str, depth + 1)));
                if i < entries.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&current_indent);
            result.push('}');
            result
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let mut result = "[\n".to_string();
            for (i, val) in arr.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format_value(val, indent_str, depth + 1));
                if i < arr.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&current_indent);
            result.push(']');
            result
        }
        _ => to_string(value).unwrap_or_default(),
    }
}

/// Validate JSON string
/// Returns: 1 if valid, 0 if invalid
#[no_mangle]
pub extern "C" fn json_validate(json_str: *const c_char) -> c_int {
    unsafe {
        let input = match c_str_to_string(json_str) {
            Some(s) => s,
            None => return 0,
        };

        match from_str::<Value>(&input) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Get value from JSON by key path (e.g., "user.name")
/// Returns: JSON value as string, or null
#[no_mangle]
pub extern "C" fn json_get(obj_json: *const c_char, path: *const c_char) -> *mut c_char {
    unsafe {
        let json_str = match c_str_to_string(obj_json) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        // Parse JSON
        let mut value = match from_str::<Value>(&json_str) {
            Ok(v) => v,
            Err(_) => return ptr::null_mut(),
        };

        // Navigate path (split by '.')
        for key in path_str.split('.') {
            value = match value.get(key) {
                Some(v) => v.clone(),
                None => return ptr::null_mut(),
            };
        }

        // Return value as JSON string
        string_to_c_str(to_string(&value).unwrap_or_default())
    }
}

/// Get string value from JSON (unwraps quotes)
#[no_mangle]
pub extern "C" fn json_get_string(obj_json: *const c_char, key: *const c_char) -> *mut c_char {
    unsafe {
        let json_str = match c_str_to_string(obj_json) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        let key_str = match c_str_to_string(key) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        // Parse JSON
        let value = match from_str::<Value>(&json_str) {
            Ok(v) => v,
            Err(_) => return ptr::null_mut(),
        };

        // Get string value
        match value.get(&key_str).and_then(|v| v.as_str()) {
            Some(s) => string_to_c_str(s.to_string()),
            None => ptr::null_mut(),
        }
    }
}

/// Get integer value from JSON
/// Returns: Integer value, or 0 if not found/not an integer
#[no_mangle]
pub extern "C" fn json_get_int(obj_json: *const c_char, key: *const c_char) -> c_int {
    unsafe {
        let json_str = match c_str_to_string(obj_json) {
            Some(s) => s,
            None => return 0,
        };

        let key_str = match c_str_to_string(key) {
            Some(s) => s,
            None => return 0,
        };

        // Parse JSON
        let value = match from_str::<Value>(&json_str) {
            Ok(v) => v,
            Err(_) => return 0,
        };

        // Get integer value
        value.get(&key_str)
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as c_int
    }
}

/// Merge two JSON objects
#[no_mangle]
pub extern "C" fn json_merge(json1: *const c_char, json2: *const c_char) -> *mut c_char {
    unsafe {
        let str1 = match c_str_to_string(json1) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        let str2 = match c_str_to_string(json2) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        // Parse both
        let mut val1 = match from_str::<Value>(&str1) {
            Ok(v) => v,
            Err(_) => return ptr::null_mut(),
        };

        let val2 = match from_str::<Value>(&str2) {
            Ok(v) => v,
            Err(_) => return ptr::null_mut(),
        };

        // Merge objects
        if let (Some(obj1), Some(obj2)) = (val1.as_object_mut(), val2.as_object()) {
            for (k, v) in obj2 {
                obj1.insert(k.clone(), v.clone());
            }
        }

        string_to_c_str(to_string(&val1).unwrap_or_default())
    }
}

/// Free a C string allocated by this library
#[no_mangle]
pub extern "C" fn json_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Plugin initialization (called when module loads)
#[no_mangle]
pub extern "C" fn wpp_plugin_init() -> c_int {
    println!("✨ W++ JSON library v1.0.0 initialized");
    0
}
