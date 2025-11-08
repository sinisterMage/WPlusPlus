// Raython Validation Framework
// Rails-like validation system for W++ models

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Mutex;

// Global validation error storage
static VALIDATION_ERRORS: Mutex<Option<HashMap<String, Vec<String>>>> = Mutex::new(None);
static ERROR_COUNTER: Mutex<usize> = Mutex::new(0);

// Initialize global state
fn ensure_init() {
    let mut errors = VALIDATION_ERRORS.lock().unwrap();
    if errors.is_none() {
        *errors = Some(HashMap::new());
    }
}

// ============================================================================
// ERROR COLLECTION MANAGEMENT
// ============================================================================

/// Create a new validation error collection and return its ID
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_create() -> *mut i8 {
    ensure_init();

    let mut counter = ERROR_COUNTER.lock().unwrap();
    *counter += 1;
    let id = format!("validation_{}", *counter);

    let mut errors = VALIDATION_ERRORS.lock().unwrap();
    if let Some(ref mut map) = *errors {
        map.insert(id.clone(), Vec::new());
    }

    CString::new(id).unwrap().into_raw()
}

/// Add an error message to a validation collection
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_add(id: *const i8, field: *const i8, message: *const i8) -> i32 {
    unsafe {
        if id.is_null() || field.is_null() || message.is_null() {
            return 0;
        }

        ensure_init();

        let id_str = CStr::from_ptr(id).to_str().unwrap_or("");
        let field_str = CStr::from_ptr(field).to_str().unwrap_or("");
        let message_str = CStr::from_ptr(message).to_str().unwrap_or("");

        let mut errors = VALIDATION_ERRORS.lock().unwrap();
        if let Some(ref mut map) = *errors {
            if let Some(error_list) = map.get_mut(id_str) {
                let error_msg = format!("{}: {}", field_str, message_str);
                error_list.push(error_msg);
                return 1;
            }
        }
        0
    }
}

/// Check if validation collection has errors
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_has(id: *const i8) -> i32 {
    unsafe {
        if id.is_null() {
            return 0;
        }

        ensure_init();

        let id_str = CStr::from_ptr(id).to_str().unwrap_or("");
        let errors = VALIDATION_ERRORS.lock().unwrap();

        if let Some(ref map) = *errors {
            if let Some(error_list) = map.get(id_str) {
                return if error_list.is_empty() { 0 } else { 1 };
            }
        }
        0
    }
}

/// Get error count for validation collection
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_count(id: *const i8) -> i32 {
    unsafe {
        if id.is_null() {
            return 0;
        }

        ensure_init();

        let id_str = CStr::from_ptr(id).to_str().unwrap_or("");
        let errors = VALIDATION_ERRORS.lock().unwrap();

        if let Some(ref map) = *errors {
            if let Some(error_list) = map.get(id_str) {
                return error_list.len() as i32;
            }
        }
        0
    }
}

/// Get all errors as JSON string
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_get(id: *const i8) -> *mut i8 {
    unsafe {
        if id.is_null() {
            return CString::new("{\"errors\": [], \"count\": 0}").unwrap().into_raw();
        }

        ensure_init();

        let id_str = CStr::from_ptr(id).to_str().unwrap_or("");
        let errors = VALIDATION_ERRORS.lock().unwrap();

        if let Some(ref map) = *errors {
            if let Some(error_list) = map.get(id_str) {
                let errors_json: Vec<String> = error_list.iter()
                    .map(|e| format!("\"{}\"", e.replace('"', "\\\"")))
                    .collect();
                let json = format!("{{\"errors\": [{}], \"count\": {}}}",
                    errors_json.join(", "),
                    error_list.len());
                return CString::new(json).unwrap().into_raw();
            }
        }
        CString::new("{\"errors\": [], \"count\": 0}").unwrap().into_raw()
    }
}

/// Clear all errors in a validation collection
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_clear(id: *const i8) -> i32 {
    unsafe {
        if id.is_null() {
            return 0;
        }

        ensure_init();

        let id_str = CStr::from_ptr(id).to_str().unwrap_or("");
        let mut errors = VALIDATION_ERRORS.lock().unwrap();

        if let Some(ref mut map) = *errors {
            if let Some(error_list) = map.get_mut(id_str) {
                error_list.clear();
                return 1;
            }
        }
        0
    }
}

/// Destroy a validation error collection
#[unsafe(no_mangle)]
pub extern "C" fn validation_errors_destroy(id: *const i8) -> i32 {
    unsafe {
        if id.is_null() {
            return 0;
        }

        ensure_init();

        let id_str = CStr::from_ptr(id).to_str().unwrap_or("");
        let mut errors = VALIDATION_ERRORS.lock().unwrap();

        if let Some(ref mut map) = *errors {
            if map.remove(id_str).is_some() {
                return 1;
            }
        }
        0
    }
}

// ============================================================================
// VALIDATION RULES
// ============================================================================

/// Validate presence (field is not empty)
#[unsafe(no_mangle)]
pub extern "C" fn validate_presence(value: *const i8, field: *const i8, errors_id: *const i8) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");

        if value_str.trim().is_empty() {
            let message = CString::new("cannot be blank").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate length (min and max)
#[unsafe(no_mangle)]
pub extern "C" fn validate_length(
    value: *const i8,
    field: *const i8,
    min: i32,
    max: i32,
    errors_id: *const i8
) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        let len = value_str.len() as i32;

        if min > 0 && len < min {
            let message = CString::new(format!("is too short (minimum is {} characters)", min)).unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        if max > 0 && len > max {
            let message = CString::new(format!("is too long (maximum is {} characters)", max)).unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        1
    }
}

/// Validate email format (simple check)
#[unsafe(no_mangle)]
pub extern "C" fn validate_email(value: *const i8, field: *const i8, errors_id: *const i8) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");

        // Simple email validation: contains @ and . after @
        if !value_str.contains('@') {
            let message = CString::new("is not a valid email address").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        let parts: Vec<&str> = value_str.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            let message = CString::new("is not a valid email address").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        if !parts[1].contains('.') {
            let message = CString::new("is not a valid email address").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        1
    }
}

/// Validate URL format (simple check)
#[unsafe(no_mangle)]
pub extern "C" fn validate_url(value: *const i8, field: *const i8, errors_id: *const i8) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");

        // Simple URL validation: starts with http:// or https://
        if !value_str.starts_with("http://") && !value_str.starts_with("https://") {
            let message = CString::new("is not a valid URL").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate format (simple pattern matching)
#[unsafe(no_mangle)]
pub extern "C" fn validate_format(
    value: *const i8,
    field: *const i8,
    pattern: *const i8,
    errors_id: *const i8
) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || pattern.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        let pattern_str = CStr::from_ptr(pattern).to_str().unwrap_or("");

        // Simple pattern matching - just check if value contains pattern
        if !value_str.contains(pattern_str) {
            let message = CString::new("is invalid format").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate numericality (value is a valid number)
#[unsafe(no_mangle)]
pub extern "C" fn validate_numericality(value: *const i8, field: *const i8, errors_id: *const i8) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");

        if value_str.parse::<f64>().is_err() {
            let message = CString::new("is not a number").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate number range
#[unsafe(no_mangle)]
pub extern "C" fn validate_number_range(
    value: i32,
    field: *const i8,
    min: i32,
    max: i32,
    errors_id: *const i8
) -> i32 {
    unsafe {
        if field.is_null() || errors_id.is_null() {
            return 0;
        }

        if value < min {
            let message = CString::new(format!("must be at least {}", min)).unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        if value > max {
            let message = CString::new(format!("must be at most {}", max)).unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            return 0;
        }

        1
    }
}

/// Validate inclusion (value is in allowed list)
#[unsafe(no_mangle)]
pub extern "C" fn validate_inclusion(
    value: *const i8,
    field: *const i8,
    allowed: *const i8,
    errors_id: *const i8
) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || allowed.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        let allowed_str = CStr::from_ptr(allowed).to_str().unwrap_or("");

        // Split comma-separated list
        let allowed_list: Vec<&str> = allowed_str.split(',').map(|s| s.trim()).collect();

        if !allowed_list.contains(&value_str) {
            let message = CString::new(format!("is not included in the list ({})", allowed_str)).unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate exclusion (value is not in forbidden list)
#[unsafe(no_mangle)]
pub extern "C" fn validate_exclusion(
    value: *const i8,
    field: *const i8,
    forbidden: *const i8,
    errors_id: *const i8
) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || forbidden.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        let forbidden_str = CStr::from_ptr(forbidden).to_str().unwrap_or("");

        // Split comma-separated list
        let forbidden_list: Vec<&str> = forbidden_str.split(',').map(|s| s.trim()).collect();

        if forbidden_list.contains(&value_str) {
            let message = CString::new(format!("is reserved ({})", forbidden_str)).unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate confirmation (two fields match)
#[unsafe(no_mangle)]
pub extern "C" fn validate_confirmation(
    value: *const i8,
    confirmation: *const i8,
    field: *const i8,
    errors_id: *const i8
) -> i32 {
    unsafe {
        if value.is_null() || confirmation.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        let confirmation_str = CStr::from_ptr(confirmation).to_str().unwrap_or("");

        if value_str != confirmation_str {
            let message = CString::new("doesn't match confirmation").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

/// Validate acceptance (checkbox must be checked, value is "true" or "1")
#[unsafe(no_mangle)]
pub extern "C" fn validate_acceptance(value: *const i8, field: *const i8, errors_id: *const i8) -> i32 {
    unsafe {
        if value.is_null() || field.is_null() || errors_id.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");

        if value_str != "true" && value_str != "1" && value_str != "yes" {
            let message = CString::new("must be accepted").unwrap();
            validation_errors_add(errors_id, field, message.as_ptr());
            0
        } else {
            1
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if string is blank (empty or whitespace only)
#[unsafe(no_mangle)]
pub extern "C" fn validation_is_blank(value: *const i8) -> i32 {
    unsafe {
        if value.is_null() {
            return 1;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        if value_str.trim().is_empty() { 1 } else { 0 }
    }
}

/// Get string length
#[unsafe(no_mangle)]
pub extern "C" fn validation_strlen(value: *const i8) -> i32 {
    unsafe {
        if value.is_null() {
            return 0;
        }

        let value_str = CStr::from_ptr(value).to_str().unwrap_or("");
        value_str.len() as i32
    }
}
