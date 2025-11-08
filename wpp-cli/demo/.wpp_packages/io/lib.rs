use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::fs::{self, File, OpenOptions, read_dir, metadata};
use std::io::{Write, BufReader, BufRead};
use std::path::Path;
use std::ptr;

// ========================================
// Helper Functions
// ========================================

unsafe fn c_str_to_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        return None;
    }
    CStr::from_ptr(c_str)
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

fn string_to_c_str(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// ========================================
// File Reading Functions
// ========================================

/// Read entire file contents as string
#[no_mangle]
pub extern "C" fn io_read_file(path: *const c_char) -> *mut c_char {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match fs::read_to_string(&path_str) {
            Ok(contents) => string_to_c_str(contents),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Read file contents as bytes and return as string (for binary files)
#[no_mangle]
pub extern "C" fn io_read_bytes(path: *const c_char) -> *mut c_char {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match fs::read(&path_str) {
            Ok(bytes) => {
                // Convert bytes to hex string representation
                let hex = bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join("");
                string_to_c_str(hex)
            }
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Read file line by line (returns newline-separated string)
#[no_mangle]
pub extern "C" fn io_read_lines(path: *const c_char) -> *mut c_char {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        let file = match File::open(&path_str) {
            Ok(f) => f,
            Err(_) => return ptr::null_mut(),
        };

        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines()
            .filter_map(|l| l.ok())
            .collect();

        string_to_c_str(lines.join("\n"))
    }
}

// ========================================
// File Writing Functions
// ========================================

/// Write string to file (overwrites existing)
#[no_mangle]
pub extern "C" fn io_write_file(path: *const c_char, content: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        let content_str = match c_str_to_string(content) {
            Some(s) => s,
            None => return 0,
        };

        match fs::write(&path_str, content_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Append string to file
#[no_mangle]
pub extern "C" fn io_append_file(path: *const c_char, content: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        let content_str = match c_str_to_string(content) {
            Some(s) => s,
            None => return 0,
        };

        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path_str) {
            Ok(f) => f,
            Err(_) => return 0,
        };

        match file.write_all(content_str.as_bytes()) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Write bytes to file (from hex string)
#[no_mangle]
pub extern "C" fn io_write_bytes(path: *const c_char, hex_data: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        let hex_str = match c_str_to_string(hex_data) {
            Some(s) => s,
            None => return 0,
        };

        // Convert hex string to bytes
        let bytes: Vec<u8> = (0..hex_str.len())
            .step_by(2)
            .filter_map(|i| {
                hex_str.get(i..i+2)
                    .and_then(|s| u8::from_str_radix(s, 16).ok())
            })
            .collect();

        match fs::write(&path_str, bytes) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

// ========================================
// File Operations
// ========================================

/// Check if file or directory exists
#[no_mangle]
pub extern "C" fn io_exists(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        if Path::new(&path_str).exists() { 1 } else { 0 }
    }
}

/// Delete file
#[no_mangle]
pub extern "C" fn io_delete_file(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match fs::remove_file(&path_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Copy file
#[no_mangle]
pub extern "C" fn io_copy_file(src: *const c_char, dst: *const c_char) -> c_int {
    unsafe {
        let src_str = match c_str_to_string(src) {
            Some(s) => s,
            None => return 0,
        };

        let dst_str = match c_str_to_string(dst) {
            Some(s) => s,
            None => return 0,
        };

        match fs::copy(&src_str, &dst_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Rename/move file
#[no_mangle]
pub extern "C" fn io_rename_file(old: *const c_char, new: *const c_char) -> c_int {
    unsafe {
        let old_str = match c_str_to_string(old) {
            Some(s) => s,
            None => return 0,
        };

        let new_str = match c_str_to_string(new) {
            Some(s) => s,
            None => return 0,
        };

        match fs::rename(&old_str, &new_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Get file size in bytes
#[no_mangle]
pub extern "C" fn io_file_size(path: *const c_char) -> i64 {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return -1,
        };

        match metadata(&path_str) {
            Ok(meta) => meta.len() as i64,
            Err(_) => -1,
        }
    }
}

/// Check if path is a file
#[no_mangle]
pub extern "C" fn io_is_file(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match metadata(&path_str) {
            Ok(meta) => if meta.is_file() { 1 } else { 0 },
            Err(_) => 0,
        }
    }
}

/// Check if path is a directory
#[no_mangle]
pub extern "C" fn io_is_dir(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match metadata(&path_str) {
            Ok(meta) => if meta.is_dir() { 1 } else { 0 },
            Err(_) => 0,
        }
    }
}

// ========================================
// Directory Operations
// ========================================

/// Create directory
#[no_mangle]
pub extern "C" fn io_create_dir(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match fs::create_dir(&path_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Create directory and all parent directories
#[no_mangle]
pub extern "C" fn io_create_dir_all(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match fs::create_dir_all(&path_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Remove empty directory
#[no_mangle]
pub extern "C" fn io_remove_dir(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match fs::remove_dir(&path_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// Remove directory and all contents
#[no_mangle]
pub extern "C" fn io_remove_dir_all(path: *const c_char) -> c_int {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return 0,
        };

        match fs::remove_dir_all(&path_str) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}

/// List directory contents (returns comma-separated list)
#[no_mangle]
pub extern "C" fn io_list_dir(path: *const c_char) -> *mut c_char {
    unsafe {
        let path_str = match c_str_to_string(path) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match read_dir(&path_str) {
            Ok(entries) => {
                let names: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        e.file_name().to_str().map(|s| s.to_string())
                    })
                    .collect();

                string_to_c_str(names.join(","))
            }
            Err(_) => ptr::null_mut(),
        }
    }
}

// ========================================
// Memory Management
// ========================================

/// Free a C string allocated by this library
#[no_mangle]
pub extern "C" fn io_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Plugin initialization
#[no_mangle]
pub extern "C" fn wpp_plugin_init() -> c_int {
    println!("📁 W++ File I/O library v1.0.0 initialized");
    0
}
