use inkwell::execution_engine::ExecutionEngine;
use once_cell::sync::{Lazy, OnceCell};
use std::{
    collections::VecDeque,
    mem,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
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

/// === ENGINE ===
pub fn set_engine(engine: ExecutionEngine<'static>) {
    let boxed = Box::new(engine);
    let ptr = Box::into_raw(boxed);
    ENGINE
        .set(EnginePtr(ptr))
        .ok()
        .expect("ENGINE already initialized");
    println!("🧠 [runtime] ENGINE stored globally");
}

unsafe fn get_engine<'a>() -> &'a ExecutionEngine<'static> {
    let EnginePtr(ptr) = ENGINE.get().expect("ENGINE not initialized");
    &**ptr
}

/// === SPAWN ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_spawn(ptr: *const ()) {
    if ptr.is_null() {
        println!("⚠️ [runtime] wpp_spawn received null pointer");
        return;
    }

    println!("🚀 [runtime] Spawning async task {:?}", ptr);

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
    println!("😴 [runtime] Yield requested");

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
pub extern "C" fn wpp_return(value: i32) {
    println!("✅ [runtime] Async function returned {value}");

    let mut queue = TASK_QUEUE.lock().unwrap();
    if let Some(task) = queue.front() {
        task.mark_finished(value);
    }

    *LAST_RESULT.lock().unwrap() = Some(value);

    // Remove finished tasks
    queue.retain(|t| !t.is_finished());
    drop(queue);

    // Continue scheduling in the background
    thread::spawn(|| {
        schedule_next();
    });
}

/// === GET LAST RESULT ===
#[unsafe(no_mangle)]
pub extern "C" fn wpp_get_last_result() -> i32 {
    let res = *LAST_RESULT.lock().unwrap();
    let val = res.unwrap_or(0);
    println!("📦 [runtime] Fetched last async result = {}", val);
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
        println!("✅ [runtime] No more tasks to run");
        return;
    }

    let task = queue.pop_front().unwrap();

    if task.is_finished() {
        let val = task.result.lock().unwrap().unwrap_or(0);
        println!("🎯 [runtime] Task {:?} finished with {}", task.func, val);
        return;
    }

    println!("🔁 [runtime] Running task {:?}", task.func);

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
    println!("🧹 [runtime] Scheduler cleared all tasks, shutdown complete");
}
pub use crate::runtime::server::{register_endpoint, wpp_start_server};

static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
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
    println!("🕓 [runtime] Waiting for async background tasks (press Ctrl+C to stop)...");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
