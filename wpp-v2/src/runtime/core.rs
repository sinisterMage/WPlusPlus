use inkwell::execution_engine::ExecutionEngine;
use once_cell::sync::{Lazy, OnceCell};
use std::{
    collections::VecDeque, ffi::c_void, mem, sync::{Arc, Mutex}, thread, time::Duration
};
use tokio::runtime::Runtime;


/// === SAFETY WRAPPERS ===
#[derive(Clone, Copy)]
struct EnginePtr(*mut ExecutionEngine<'static>);
unsafe impl Send for EnginePtr {}
unsafe impl Sync for EnginePtr {}

/// === TASK STRUCT ===
#[derive(Debug)]
pub struct Task {
    pub func: *const (),
    pub result: Mutex<Option<i32>>,
    pub finished: Mutex<bool>,
}

impl Task {
    pub fn new(func: *const ()) -> Arc<Self> {
        Arc::new(Self {
            func,
            result: Mutex::new(None),
            finished: Mutex::new(false),
        })
    }

    pub fn mark_finished(&self, val: i32) {
        *self.result.lock().unwrap() = Some(val);
        *self.finished.lock().unwrap() = true;
    }

    pub fn is_finished(&self) -> bool {
        *self.finished.lock().unwrap()
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

/// === GLOBALS ===
static ENGINE: OnceCell<EnginePtr> = OnceCell::new();
static TASK_QUEUE: Lazy<Mutex<VecDeque<Arc<Task>>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
static LAST_RESULT: Lazy<Mutex<Option<i32>>> = Lazy::new(|| Mutex::new(None));

fn debug_enabled() -> bool {
    std::env::var("WPP_DEBUG").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false)
}

/// === ENGINE ===
pub fn set_engine(engine: ExecutionEngine<'static>) {
    let boxed = Box::new(engine);
    let ptr = Box::into_raw(boxed);

    if ENGINE.set(EnginePtr(ptr)).is_err() {
        if debug_enabled() { println!("⚠️ [runtime] ENGINE was already initialized, keeping existing engine"); }
        // Clean up the new engine we tried to set
        unsafe { drop(Box::from_raw(ptr)); }
    } else {
        if debug_enabled() { println!("🧠 [runtime] ENGINE stored globally"); }
    }
}

pub unsafe fn get_engine<'a>() -> &'a ExecutionEngine<'static> {
    let EnginePtr(ptr) = *ENGINE.get().expect("ENGINE not initialized");
    let nn = std::ptr::NonNull::new(ptr).expect("ENGINE pointer was null");
    let r: &ExecutionEngine<'static> = unsafe { nn.as_ref() };
    r
}

/// === SPAWN ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_spawn(ptr: *const ()) {
    if ptr.is_null() {
        if debug_enabled() { println!("⚠️ [runtime] wpp_spawn received null pointer"); }
        return;
    }

    if debug_enabled() { println!("🚀 [runtime] Spawning async task {:?}", ptr); }

    let task = Task::new(ptr);
    TASK_QUEUE.lock().unwrap().push_back(task);

    // schedule asynchronously
    thread::spawn(|| {
        schedule_next();
    });
}

/// === YIELD ===
/// Yield cooperatively without blocking caller
#[unsafe(no_mangle)]
pub extern "C" fn wpp_yield() {
    if debug_enabled() { println!("😴 [runtime] Yield requested"); }

    // Move current task to the back of the queue
    let mut queue = TASK_QUEUE.lock().unwrap();
    if let Some(task) = queue.pop_front() {
        if !task.is_finished() {
            queue.push_back(task);
        }
    }
    drop(queue);

    // Run scheduler asynchronously, non-blocking
    thread::spawn(|| {
        schedule_next();
    });
}

/// === RETURN ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_return(value: *const c_void, type_tag: i32) {
    if debug_enabled() {
        unsafe {
            match type_tag {
                1 => {
                    let ptr = value as *const i32;
                    println!("✅ [runtime] Returned int: {}", *ptr);
                }
                2 => {
                    let ptr = value as *const f32;
                    println!("✅ [runtime] Returned float: {}", *ptr);
                }
                3 => {
                    let ptr = value as *const bool;
                    println!("✅ [runtime] Returned bool: {}", *ptr);
                }
                4 => {
                    let ptr = value as *const *const i8;
                    println!("✅ [runtime] Returned string pointer: {:?}", *ptr);
                }
                _ => println!("⚠️ [runtime] Unknown return type tag: {type_tag}"),
            }
        }
    }

    // ✅ continue async scheduling
    let mut queue = TASK_QUEUE.lock().unwrap();
    if let Some(task) = queue.front() {
        task.mark_finished(0);
    }
    queue.retain(|t| !t.is_finished());
    drop(queue);

    thread::spawn(|| {
        schedule_next();
    });
}

/// === GET LAST RESULT ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_get_last_result() -> i32 {
    let res = *LAST_RESULT.lock().unwrap();
    let val = res.unwrap_or(0);
    if debug_enabled() { println!("📦 [runtime] Fetched last async result = {}", val); }
    val
}

/// === DYNAMIC SLEEP ===
fn dynamic_sleep() {
    let len = TASK_QUEUE.lock().unwrap().len();

    match len {
        0 => {}
        1..=3 => thread::sleep(Duration::from_millis(1)),
        4..=10 => thread::sleep(Duration::from_micros(200)),
        _ => {} // no sleep for large queues
    }
}

/// === SCHEDULER ===
fn schedule_next() {
    let mut queue = TASK_QUEUE.lock().unwrap();

    if queue.is_empty() {
        if debug_enabled() { println!("✅ [runtime] No more tasks to run"); }
        return;
    }

    let task = queue.pop_front().unwrap();

    if task.is_finished() {
        let val = task.result.lock().unwrap().unwrap_or(0);
        if debug_enabled() { println!("🎯 [runtime] Task {:?} finished with {}", task.func, val); }
        return;
    }

    if debug_enabled() { println!("🔁 [runtime] Running task {:?}", task.func); }

    unsafe {
        let func: extern "C" fn() = mem::transmute(task.func);
        drop(queue);
        func();
    }

    dynamic_sleep();

    // Only requeue if unfinished
    let mut queue = TASK_QUEUE.lock().unwrap();
    if !task.is_finished() {
        queue.push_back(task);
    }

    drop(queue);

    // Schedule next task asynchronously (non-blocking)
    thread::spawn(|| schedule_next());
}

/// === OPTIONAL CLEAN SHUTDOWN ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_shutdown() {
    TASK_QUEUE.lock().unwrap().clear();
    if debug_enabled() { println!("🧹 [runtime] Scheduler cleared all tasks, shutdown complete"); }
}
pub use crate::runtime::server::{register_endpoint, wpp_start_server};

static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus) // ✅ Match CPU count (Phase 1 Optimization #2)
        .thread_name("wpp-http-worker")
        .thread_stack_size(2 * 1024 * 1024) // 2MB stack
        .enable_all()
        .max_blocking_threads(512) // Increase from default
        .event_interval(61) // Tune for latency/throughput balance
        .global_queue_interval(31) // Check global queue less frequently
        .build()
        .expect("❌ Failed to build Tokio runtime")
});

/// Register an async task inside the shared runtime
pub fn register_task<F>(fut: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    TOKIO_RT.spawn(fut);
}
#[unsafe(no_mangle)]
pub extern "C" fn wpp_runtime_wait() {
    if debug_enabled() { println!("🕓 [runtime] Waiting for async background tasks (press Ctrl+C to stop)..."); }
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_concat(a: *const i8, b: *const i8) -> *mut i8 {
    unsafe {
        if a.is_null() || b.is_null() {
            return std::ptr::null_mut();
        }

        let sa = std::ffi::CStr::from_ptr(a).to_str().unwrap_or("");
        let sb = std::ffi::CStr::from_ptr(b).to_str().unwrap_or("");

        let concat = format!("{}{}", sa, sb);
        let cstring = std::ffi::CString::new(concat).unwrap();
        cstring.into_raw()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_substr(s: *const i8, start: i32, length: i32) -> *mut i8 {
    unsafe {
        if s.is_null() {
            return std::ptr::null_mut();
        }

        let string = std::ffi::CStr::from_ptr(s).to_str().unwrap_or("");
        let start = start.max(0) as usize;
        let length = length.max(0) as usize;

        let result = if start < string.len() {
            let end = (start + length).min(string.len());
            &string[start..end]
        } else {
            ""
        };

        let cstring = std::ffi::CString::new(result).unwrap();
        cstring.into_raw()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_index_of(haystack: *const i8, needle: *const i8) -> i32 {
    unsafe {
        if haystack.is_null() || needle.is_null() {
            return -1;
        }

        let hay = std::ffi::CStr::from_ptr(haystack).to_str().unwrap_or("");
        let need = std::ffi::CStr::from_ptr(needle).to_str().unwrap_or("");

        match hay.find(need) {
            Some(pos) => pos as i32,
            None => -1,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_replace(s: *const i8, find: *const i8, replace: *const i8) -> *mut i8 {
    unsafe {
        if s.is_null() || find.is_null() || replace.is_null() {
            return std::ptr::null_mut();
        }

        let string = std::ffi::CStr::from_ptr(s).to_str().unwrap_or("");
        let find_str = std::ffi::CStr::from_ptr(find).to_str().unwrap_or("");
        let replace_str = std::ffi::CStr::from_ptr(replace).to_str().unwrap_or("");

        let result = string.replace(find_str, replace_str);
        let cstring = std::ffi::CString::new(result).unwrap();
        cstring.into_raw()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_to_upper(s: *const i8) -> *mut i8 {
    unsafe {
        if s.is_null() {
            return std::ptr::null_mut();
        }

        let string = std::ffi::CStr::from_ptr(s).to_str().unwrap_or("");
        let result = string.to_uppercase();
        let cstring = std::ffi::CString::new(result).unwrap();
        cstring.into_raw()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_to_lower(s: *const i8) -> *mut i8 {
    unsafe {
        if s.is_null() {
            return std::ptr::null_mut();
        }

        let string = std::ffi::CStr::from_ptr(s).to_str().unwrap_or("");
        let result = string.to_lowercase();
        let cstring = std::ffi::CString::new(result).unwrap();
        cstring.into_raw()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_str_trim(s: *const i8) -> *mut i8 {
    unsafe {
        if s.is_null() {
            return std::ptr::null_mut();
        }

        let string = std::ffi::CStr::from_ptr(s).to_str().unwrap_or("");
        let result = string.trim();
        let cstring = std::ffi::CString::new(result).unwrap();
        cstring.into_raw()
    }
}
