use inkwell::execution_engine::ExecutionEngine;
use once_cell::sync::{Lazy, OnceCell};
use std::{
    collections::VecDeque,
    mem,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

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
static TASK_QUEUE: Lazy<Mutex<VecDeque<Arc<Task>>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));
static LAST_RESULT: Lazy<Mutex<Option<i32>>> = Lazy::new(|| Mutex::new(None));

/// === ENGINE ===
/// Store the LLVM JIT engine globally
pub fn set_engine(engine: ExecutionEngine<'static>) {
    let boxed = Box::new(engine);
    let ptr = Box::into_raw(boxed);
    ENGINE
        .set(EnginePtr(ptr))
        .ok()
        .expect("ENGINE already initialized");
    println!("🧠 [runtime] ENGINE stored globally");
}

/// Access engine unsafely (global lifetime)
unsafe fn get_engine<'a>() -> &'a ExecutionEngine<'static> {
    let EnginePtr(ptr) = ENGINE.get().expect("ENGINE not initialized");
    &**ptr
}

/// === SPAWN ===
/// Called from LLVM when starting an async function
#[unsafe(no_mangle)]
pub extern "C" fn wpp_spawn(ptr: *const ()) {
    if ptr.is_null() {
        println!("⚠️ [runtime] wpp_spawn received null pointer");
        return;
    }

    println!("🚀 [runtime] Spawning async task {:?}", ptr);

    let task = Task::new(ptr);
    TASK_QUEUE.lock().unwrap().push_back(task);

    schedule_next();
}

/// === YIELD ===
/// Called from LLVM when async function hits `await`
#[unsafe(no_mangle)]
pub extern "C" fn wpp_yield() {
    println!("😴 [runtime] Yielding control...");
    thread::sleep(Duration::from_millis(5));
    schedule_next();
}

/// === RETURN ===
/// Called from LLVM when async func returns a value
#[unsafe(no_mangle)]
pub extern "C" fn wpp_return(value: i32) {
    println!("✅ [runtime] Async function returned {value}");

    let mut queue = TASK_QUEUE.lock().unwrap();
    if let Some(task) = queue.front() {
        task.mark_finished(value);
    }

    *LAST_RESULT.lock().unwrap() = Some(value);
    drop(queue);

    schedule_next();
}

/// === GET LAST RESULT ===
/// Allows awaiters to read last async return value
#[unsafe(no_mangle)]
pub extern "C" fn wpp_get_last_result() -> i32 {
    let res = *LAST_RESULT.lock().unwrap();
    res.unwrap_or(0)
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
        println!("🎯 [runtime] Task {:?} already finished with value {}", task.func, val);
        return;
    }

    println!("🔁 [runtime] Running task {:?}", task.func);

    unsafe {
        let func: extern "C" fn() = mem::transmute(task.func);
        drop(queue);
        func();
    }

    thread::sleep(Duration::from_millis(5));
}
