use inkwell::execution_engine::ExecutionEngine;
use once_cell::sync::OnceCell;
use std::{
    collections::VecDeque,
    ffi::c_void,
    mem,
    sync::Mutex,
    thread,
    time::Duration,
};

/// === SAFETY WRAPPERS ===
/// We wrap raw pointers in types that are Send + Sync so OnceCell can store them safely.
#[derive(Clone, Copy)]
pub struct EnginePtr(*mut ExecutionEngine<'static>);
unsafe impl Send for EnginePtr {}
unsafe impl Sync for EnginePtr {}

#[derive(Clone, Copy)]
pub struct TaskPtr(*const c_void);
unsafe impl Send for TaskPtr {}
unsafe impl Sync for TaskPtr {}

/// === Global Execution Engine (thread-safe, stored as raw pointer) ===
static ENGINE: OnceCell<EnginePtr> = OnceCell::new();

/// === Cooperative Task Scheduler ===
static TASK_QUEUE: once_cell::sync::Lazy<Mutex<VecDeque<TaskPtr>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(VecDeque::new()));

/// Store the engine globally (called from run_jit)
pub fn set_engine(engine: ExecutionEngine<'static>) {
    let boxed = Box::new(engine);
    let ptr = Box::into_raw(boxed);
    ENGINE.set(EnginePtr(ptr)).ok().expect("ENGINE already initialized");
    println!("🧠 [runtime] ENGINE stored globally");
}

/// Get the current engine (unsafe, global lifetime)
pub unsafe fn get_engine<'a>() -> &'a ExecutionEngine<'static> {
    let EnginePtr(ptr) = ENGINE.get().expect("ENGINE not initialized before runtime call");
    &**ptr
}

/// === wpp_spawn(ptr) ===
/// Called from LLVM to start or queue async tasks
#[unsafe(no_mangle)]
pub extern "C" fn wpp_spawn(ptr: *const c_void) {
    if ptr.is_null() {
        println!("⚠️ [runtime] Spawn called with null pointer");
        return;
    }

    println!("🚀 [runtime] Spawning async task {:?}", ptr);
    {
        let mut queue = TASK_QUEUE.lock().unwrap();
        queue.push_back(TaskPtr(ptr));
    }

    schedule_next();
}

/// === wpp_yield() ===
/// Called from LLVM when an async func hits `await`
#[unsafe(no_mangle)]
pub extern "C" fn wpp_yield() {
    println!("😴 [runtime] Yielding control to next task...");
    thread::sleep(Duration::from_millis(10));
    schedule_next();
}

/// === Scheduler ===
/// Runs next queued task safely
fn schedule_next() {
    let mut queue = TASK_QUEUE.lock().unwrap();
    if queue.is_empty() {
        println!("✅ [runtime] No more tasks to run");
        return;
    }

    let TaskPtr(ptr) = queue.pop_front().unwrap();
    println!("🔁 [runtime] Running task {:?}", ptr);

    unsafe {
        if ptr.is_null() {
            println!("🚨 [runtime] NULL pointer in scheduler!");
            return;
        }

        // Correct function pointer cast
        let func: extern "C" fn() = mem::transmute::<*const c_void, extern "C" fn()>(ptr);

        drop(queue); // release lock before running
        func(); // execute async task
    }

    thread::sleep(Duration::from_millis(5));
}
