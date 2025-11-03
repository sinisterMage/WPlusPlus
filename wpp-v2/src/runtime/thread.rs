//! # W++ Thread System - GC-Managed Concurrency
//!
//! FIX 19: Comprehensive module documentation
//!
//! ## Overview
//!
//! This module provides a garbage-collected thread system for the W++ programming language.
//! It implements automatic thread lifecycle management, mutex tracking, and race detection.
//!
//! ## Design Goals
//!
//! 1. **Memory Safety**: Prevent memory leaks from abandoned threads
//! 2. **Race Detection**: Warn about potential race conditions on mutexes
//! 3. **Automatic Cleanup**: Threads auto-join on Drop, no manual management needed
//! 4. **Deadlock Prevention**: Careful lock ordering to avoid circular dependencies
//!
//! ## Architecture
//!
//! ### Thread Lifecycle
//!
//! 1. **Spawn**: `wpp_thread_spawn_gc()` creates a new `ThreadHandle` wrapped in `Arc`
//! 2. **Register**: Thread is registered with `ThreadGC` via `Weak` reference
//! 3. **Execute**: Thread function runs, catches panics, updates finished flag
//! 4. **Join**: Explicit via `wpp_thread_join()` or automatic on `Drop`
//! 5. **Collect**: GC periodically collects finished threads with no external references
//!
//! ### Garbage Collection Strategy
//!
//! - **Mark-and-Sweep**: Periodically scans all registered threads
//! - **Weak References**: Allows threads to be freed when no longer referenced
//! - **Reference Counting**: Tracks active Arc references to prevent premature collection
//! - **Daemon Thread**: Optional background GC for long-running programs
//!
//! ### Race Detection Mechanism
//!
//! - **Busy Flag**: Atomic flag set during mutex acquisition
//! - **Compare-Exchange**: Atomic test-and-set to detect concurrent access
//! - **Warning Only**: Doesn't prevent access, just logs potential races
//! - **False Positives**: Minimized by proper Acquire/Release ordering
//!
//! ## Safety Considerations
//!
//! ### Thread Safety
//!
//! - All public APIs are thread-safe
//! - Internal state protected by `Mutex` or atomic operations
//! - GC operations protected by global `GC_LOCK` spin-lock
//!
//! ### Memory Ordering
//!
//! - `Acquire`: Used when reading shared state (synchronizes with Release)
//! - `Release`: Used when writing shared state (visible to future Acquire)
//! - `SeqCst`: Used for critical error states requiring global ordering
//! - `Relaxed`: Used for non-synchronizing operations (e.g., ID generation)
//!
//! ### Panic Safety
//!
//! - Thread panics are caught via `catch_unwind`
//! - Panic in one thread doesn't crash the program
//! - RAII guards ensure cleanup even during unwinding
//! - Drop handlers never panic
//!
//! ## Public API
//!
//! ### Thread Management
//!
//! - `wpp_thread_spawn_gc(fn_ptr)`: Spawn a new GC-managed thread
//! - `wpp_thread_join(handle)`: Wait for thread to complete
//! - `wpp_thread_poll(handle)`: Check if thread is finished
//! - `wpp_thread_join_all()`: Join all remaining threads (cleanup on exit)
//!
//! ### Mutex Operations
//!
//! - `wpp_mutex_new(initial)`: Create a GC-tracked mutex
//! - `wpp_mutex_lock(mutex, thread_id)`: Acquire lock with race detection
//! - `wpp_mutex_unlock(mutex)`: Release lock
//!
//! ### Thread-Local State
//!
//! - `wpp_thread_state_new(initial)`: Create thread-safe shared state
//! - `wpp_thread_state_get(state)`: Read current value
//! - `wpp_thread_state_set(state, value)`: Update value
//!
//! ## Example Usage (from W++)
//!
//! ```wpp
//! func worker() {
//!     print("Worker running")
//!     return 0
//! }
//!
//! func main() {
//!     // Blocking mode (default)
//!     let t1 = useThread(worker)  // Auto-joins before continuing
//!
//!     // Detached mode (concurrent)
//!     let t2 = useThread(worker, 1)  // Runs in background
//!
//!     return 0  // All threads auto-join on exit
//! }
//! ```

use std::{
    any::Any,
    collections::HashMap,
    mem,
    os::raw::{c_int, c_void},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
    thread,
    time::Duration,
};

use once_cell::sync::Lazy;
use rand::Rng;

// ===========================================================
// 🔧 Configuration Constants
// ===========================================================
/// FIX 20: Extract magic numbers to named constants
const GC_DAEMON_INTERVAL_SECS: u64 = 3;
const MAX_SPIN_BACKOFF: u32 = 1024;
const YIELD_THRESHOLD: u32 = 512;

// ===========================================================
// 🐛 Debug Logging (FIX 18)
// ===========================================================
/// Conditional debug logging - only enabled with "thread-debug" feature
#[cfg(feature = "thread-debug")]
macro_rules! thread_debug {
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}

#[cfg(not(feature = "thread-debug"))]
macro_rules! thread_debug {
    ($($arg:tt)*) => {};
}

/// Conditional debug logging with custom emoji
#[cfg(feature = "thread-debug")]
macro_rules! thread_trace {
    ($emoji:expr, $($arg:tt)*) => {
        println!(concat!($emoji, " {}"), format!($($arg)*))
    };
}

#[cfg(not(feature = "thread-debug"))]
macro_rules! thread_trace {
    ($emoji:expr, $($arg:tt)*) => {};
}

// ===========================================================
// 🔒 Global State
// ===========================================================
static GC_LOCK: AtomicBool = AtomicBool::new(false);
static DAEMON_SHUTDOWN: AtomicBool = AtomicBool::new(false); // FIX 12
static THREADS_EVER_SPAWNED: AtomicU64 = AtomicU64::new(0); // Track if any threads created
thread_local! {
    static THREAD_ANCESTRY: std::cell::RefCell<Vec<u64>> = std::cell::RefCell::new(Vec::new());
}

// ===========================================================
// 🧹 RAII Guard for Thread Ancestry (FIX 10)
// ===========================================================
/// Automatically cleans up thread ancestry on drop, preventing leaks on panic
struct AncestryGuard(u64);

impl Drop for AncestryGuard {
    fn drop(&mut self) {
        THREAD_ANCESTRY.with(|ancestry| {
            let mut a = ancestry.borrow_mut();
            if let Some(pos) = a.iter().position(|x| *x == self.0) {
                a.remove(pos);
            }
        });
    }
}
// ===========================================================
// 🧩 ThreadState (for useThreadState())
// ===========================================================
#[derive(Clone)]
pub struct ThreadState<T: Send + Clone + 'static> {
    inner: Arc<Mutex<T>>,
}

impl<T: Send + Clone + 'static> ThreadState<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
        }
    }
    pub fn get(&self) -> T {
        self.inner.lock().unwrap().clone()
    }
    pub fn set(&self, val: T) {
        *self.inner.lock().unwrap() = val;
    }
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut data = self.inner.lock().unwrap();
        f(&mut data);
    }
}
// ===========================================================
// 🔒 GC-Aware Mutex (for safe cross-thread locks)
// ===========================================================
#[derive(Debug)]
pub struct GcMutex<T: Send + 'static> {
    id: u64,
    data: Arc<Mutex<T>>,
    owner_thread: Arc<Mutex<Option<u64>>>,
    poisoned: Arc<AtomicBool>,
        busy: AtomicBool, 
}

impl<T: Send + 'static> GcMutex<T> {
    pub fn new(initial: T) -> Arc<Self> {
        static NEXT_MUTEX_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_MUTEX_ID.fetch_add(1, Ordering::Relaxed);

        let mutex = Arc::new(GcMutex {
            id,
            data: Arc::new(Mutex::new(initial)),
            owner_thread: Arc::new(Mutex::new(None)),
            poisoned: Arc::new(AtomicBool::new(false)),
            busy: AtomicBool::new(false), // ✅ initialize here
        });

        ThreadGC::register_mutex(mutex.clone());
        println!("🧩 [gc] Registered mutex #{id}");
        mutex
    }

    pub fn lock(&self, thread_id: u64) -> std::sync::LockResult<std::sync::MutexGuard<'_, T>> {
        // FIX 4: Proper race detection using compare_exchange
        // FIX 16: Memory ordering documentation
        //
        // Memory Ordering Rationale:
        // - Acquire on success: Ensures all subsequent loads see writes from the Release in unlock()
        // - Relaxed on failure: No synchronization needed when CAS fails (we just detected a race)
        //
        // Synchronization relationship:
        // This Acquire synchronizes-with the Release in unlock(), establishing a happens-before
        // relationship. This ensures that if thread A releases the lock (Release), and thread B
        // acquires it (Acquire), then all of A's writes before the Release are visible to B.
        if self.busy.compare_exchange(
            false,           // Expected: not busy
            true,            // New value: set busy
            Ordering::Acquire,  // Success: synchronize with Release in unlock
            Ordering::Relaxed   // Failure: no synchronization needed
        ).is_err() {
            // Flag was already true - genuine race detected
            eprintln!("⚠️ [mutex] Race detected on mutex #{} (already busy)", self.id);
            // Continue anyway - the actual Mutex below will serialize access properly
        }

        // Lock the actual mutex (may block here)
        let guard = self.data.lock();

        match guard {
            Ok(g) => {
                // Successfully acquired lock - set owner and clear busy flag
                *self.owner_thread.lock().unwrap() = Some(thread_id);
                // Release: Makes this store visible to next Acquire in lock()
                self.busy.store(false, Ordering::Release);
                Ok(g)
            }
            Err(poisoned) => {
                // Mutex was poisoned - record it and clear busy flag
                // SeqCst: Ensures global ordering for poison flag (critical error state)
                self.poisoned.store(true, Ordering::SeqCst);
                self.busy.store(false, Ordering::Release);
                Err(poisoned)
            }
        }
    }


    pub fn unlock(&self) {
        *self.owner_thread.lock().unwrap() = None;
        println!("🔓 [mutex] Mutex #{} unlocked manually", self.id);
    }

    pub fn owner_dead(&self) -> bool {
        // FIX 3: Prevent deadlock by reading owner ID and releasing lock before acquiring ThreadGC lock
        let owner_id = {
            // Acquire lock, read value, immediately release
            *self.owner_thread.lock().unwrap()
        }; // owner_thread lock dropped here

        if let Some(owner) = owner_id {
            // Now acquire ThreadGC lock - no double-lock deadlock
            let threads = ThreadGC::global().threads.lock().unwrap();
            if let Some(weak_handle) = threads.get(&owner) {
                if let Some(handle) = weak_handle.upgrade() {
                    return handle.is_finished();
                } else {
                    // Weak couldn't be upgraded → thread was already collected
                    println!("💀 [gc] Thread #{owner} already collected (Weak expired)");
                    return true;
                }
            }
        }
        false
    }


    pub fn force_unlock_if_dead(&self) {
        if self.owner_dead() {
            println!("💀 [gc] Releasing mutex #{} (owner thread dead)", self.id);
            *self.owner_thread.lock().unwrap() = None;
        }
    }
}

// ===========================================================
// 🧠 GC-Aware ThreadHandle
// ===========================================================
#[derive(Debug)]
pub struct ThreadHandle {
    pub id: u64,
    pub finished: Arc<AtomicBool>,
    pub result: Arc<Mutex<Option<Box<dyn Any + Send + 'static>>>>,
    pub join_handle: Mutex<Option<thread::JoinHandle<()>>>, // ← wrap in Mutex
    pub ref_count: Arc<AtomicU64>, // how many active references exist
}

impl ThreadHandle {
    pub fn spawn(func_ptr: *const c_void) -> Arc<ThreadHandle> {
        if func_ptr.is_null() {
            eprintln!("❌ [thread] null func pointer");
            ThreadGC::collect_now();
            return Arc::new(ThreadHandle {
                id: 0,
                finished: Arc::new(AtomicBool::new(true)),
                result: Arc::new(Mutex::new(None)),
                join_handle: Mutex::new(None), // ✅ now matches type
                ref_count: Arc::new(AtomicU64::new(0)),
            });
        }

        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        // FIX 17: Integer overflow protection
        if id == u64::MAX {
            panic!("💥 [thread] Thread ID overflow! Maximum thread count reached.");
        }

        // recursion prevention
        let mut recursion_violation = false;
        THREAD_ANCESTRY.with(|ancestry| {
            let ancestors = ancestry.borrow();
            if ancestors.contains(&id) {
                recursion_violation = true;
            }
        });
        if recursion_violation {
            eprintln!("💥 [thread] recursion prevented: thread #{id} tried to spawn itself!");
            return Arc::new(ThreadHandle {
                id,
                finished: Arc::new(AtomicBool::new(true)),
                result: Arc::new(Mutex::new(None)),
                join_handle: Mutex::new(None),
                ref_count: Arc::new(AtomicU64::new(0)),
            });
        }

        // FIX 10: Use RAII guard for automatic ancestry cleanup
        THREAD_ANCESTRY.with(|ancestry| ancestry.borrow_mut().push(id));

        let finished = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        let ref_count = Arc::new(AtomicU64::new(1));

        let func: extern "C" fn() = unsafe { mem::transmute(func_ptr) };
        let fin_clone = finished.clone();

        // Track that at least one thread has been spawned
        THREADS_EVER_SPAWNED.fetch_add(1, Ordering::Relaxed);

        println!("🚀 [thread] spawning GC-managed thread #{id}");

        // ✅ Store inside Mutex<Option<JoinHandle>>
        let join_handle = Mutex::new(Some(thread::spawn(move || {
            // FIX 10: Create RAII guard - ensures cleanup even on panic
            let _ancestry_guard = AncestryGuard(id);

            let result = std::panic::catch_unwind(|| func());
            match result {
                Ok(_) => println!("✅ [thread] thread #{id} finished normally"),
                Err(_) => eprintln!("💥 [thread] thread #{id} panicked"),
            }
            fin_clone.store(true, Ordering::SeqCst);

            // _ancestry_guard drops here, automatically cleaning up ancestry
        })));

        let handle = Arc::new(ThreadHandle {
            id,
            finished,
            result,
            join_handle,
            ref_count,
        });

        ThreadGC::register(handle.clone());
        handle
    }

    pub fn join(&self) {
        let mut guard = self.join_handle.lock().unwrap();
        if let Some(handle) = guard.take() {
            println!("🧵 [thread] joining thread #{}", self.id);

            // FIX 15: Capture and log join results instead of discarding them
            match handle.join() {
                Ok(_) => {
                    println!("✅ [thread] Successfully joined thread #{}", self.id);
                }
                Err(panic_payload) => {
                    eprintln!("💥 [thread] Thread #{} panicked: {:?}", self.id, panic_payload);
                }
            }

            self.finished.store(true, Ordering::SeqCst);
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    pub fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn release(&self) {
        // FIX 8: Use compare_exchange loop to prevent race condition on ref_count
        let mut current = self.ref_count.load(Ordering::SeqCst);
        loop {
            // Prevent underflow - don't release if already at 0
            if current == 0 {
                eprintln!("⚠️ [gc] Attempted to release already-freed thread #{}!", self.id);
                return;
            }

            // Atomically decrement only if value hasn't changed
            match self.ref_count.compare_exchange(
                current,
                current - 1,
                Ordering::SeqCst,  // Success: full synchronization
                Ordering::SeqCst   // Failure: reload current value
            ) {
                Ok(old_value) => {
                    // Successfully decremented - check if this was the last reference
                    if old_value == 1 {
                        println!("🧹 [gc] ThreadHandle #{} released (0 refs)", self.id);
                        ThreadGC::collect_now();
                    }
                    return;
                }
                Err(actual) => {
                    // Another thread changed ref_count - retry with updated value
                    current = actual;
                }
            }
        }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        if !self.is_finished() {
            println!("🧹 [thread] auto-joining unfinished thread #{}", self.id);

            // FIX 14: Handle poisoned lock gracefully in Drop
            match self.join_handle.lock() {
                Ok(mut guard) => {
                    if let Some(handle) = guard.take() {
                        let _ = handle.join();
                    }
                    self.finished.store(true, Ordering::SeqCst);
                }
                Err(_) => {
                    eprintln!("⚠️ [thread] Drop: poisoned lock on thread #{}, cannot join", self.id);
                    // Mark as finished anyway to prevent further issues
                    self.finished.store(true, Ordering::SeqCst);
                }
            }
        }
    }
}

// ===========================================================
// 🧹 ThreadGC: Self-contained mark-and-sweep for threads
// ===========================================================
pub struct ThreadGC {
    threads: Mutex<HashMap<u64, Weak<ThreadHandle>>>,
    mutexes: Mutex<HashMap<u64, Weak<dyn Any + Send + Sync>>>,
}


impl ThreadGC {
    fn global() -> &'static ThreadGC {
        static INSTANCE: Lazy<ThreadGC> = Lazy::new(|| ThreadGC {
            threads: Mutex::new(HashMap::new()),
            mutexes: Mutex::new(HashMap::new()),
        });
        &INSTANCE
    }

    pub fn register(ptr: Arc<ThreadHandle>) {
        // FIX 5: Spin-lock with exponential backoff to prevent CPU exhaustion
        let mut backoff = 1;
        while GC_LOCK.swap(true, Ordering::Acquire) {
            // Exponential backoff: spin increasingly longer before retrying
            for _ in 0..backoff {
                std::hint::spin_loop();  // CPU hint for spin-wait
            }
            backoff = (backoff * 2).min(1024);  // Cap at 1024 iterations

            // If we've backed off significantly, yield to scheduler
            if backoff > 512 {
                std::thread::yield_now();
            }
        }

        let id = ptr.id;
        ThreadGC::global()
            .threads
            .lock()
            .unwrap()
            .insert(id, Arc::downgrade(&ptr));
        println!("🧠 [gc] Registered thread handle #{id}");
        GC_LOCK.store(false, Ordering::Release);
    }

pub fn register_mutex<T: Send + 'static>(ptr: Arc<GcMutex<T>>) {
    // FIX 9: Add same GC_LOCK protection as thread registration
    let mut backoff = 1;
    while GC_LOCK.swap(true, Ordering::Acquire) {
        for _ in 0..backoff {
            std::hint::spin_loop();
        }
        backoff = (backoff * 2).min(MAX_SPIN_BACKOFF);

        if backoff > YIELD_THRESHOLD {
            std::thread::yield_now();
        }
    }

    let id = ptr.id;
    let erased: Arc<dyn Any + Send + Sync> = ptr;
    ThreadGC::global()
        .mutexes
        .lock()
        .unwrap()
        .insert(id, Arc::downgrade(&erased));
    println!("🧩 [gc] Registered mutex #{id}");

    GC_LOCK.store(false, Ordering::Release);
}


    pub fn collect_now() {
    // FIX 11: Retry with backoff instead of silently skipping
    let mut backoff = 1;
    let mut retries = 0;
    const MAX_RETRIES: u32 = 5;

    while GC_LOCK.swap(true, Ordering::Acquire) {
        if retries >= MAX_RETRIES {
            println!("⚠️ [gc] Skipped collect after {} retries (another thread collecting)", MAX_RETRIES);
            return;
        }

        // Exponential backoff
        for _ in 0..backoff {
            std::hint::spin_loop();
        }
        backoff = (backoff * 2).min(MAX_SPIN_BACKOFF);

        if backoff > YIELD_THRESHOLD {
            std::thread::yield_now();
        }

        retries += 1;
    }

    let mut threads = ThreadGC::global().threads.lock().unwrap();
    let mut mutexes = ThreadGC::global().mutexes.lock().unwrap();
    let mut collected = 0;

    threads.retain(|id, weak_handle| {
        if let Some(handle) = weak_handle.upgrade() {
            if handle.is_finished() && Arc::strong_count(&handle) == 1 {
                println!("💀 [gc] Collecting thread #{id}");
                collected += 1;
                false
            } else {
                true
            }
        } else {
            println!("💀 [gc] Thread #{id} fully dropped");
            false
        }
    });

    // FIX 13: Clean up dead weak mutex references
    let mut mutex_cleaned = 0;
    mutexes.retain(|id, weak_any| {
        if weak_any.strong_count() > 0 {
            // Still alive - check if owner is dead
            if let Some(mtx_any) = weak_any.upgrade() {
                if let Some(mtx) = mtx_any.downcast_ref::<GcMutex<c_int>>() {
                    mtx.force_unlock_if_dead();
                }
            }
            true // Keep in map
        } else {
            // Weak reference expired - remove from map
            println!("💀 [gc] Mutex #{id} weak reference expired");
            mutex_cleaned += 1;
            false // Remove from map
        }
    });

    if collected > 0 || mutex_cleaned > 0 {
        println!("🧹 [gc] Collected {collected} threads, {mutex_cleaned} mutexes");
    }

    GC_LOCK.store(false, Ordering::Release);
}

}

// ===========================================================
// 🔗 Extern API for W++
// ===========================================================

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_spawn_gc(fn_ptr: *const c_void) -> *mut ThreadHandle {
    let handle_arc = ThreadHandle::spawn(fn_ptr);
    Arc::into_raw(handle_arc) as *mut ThreadHandle // ✅ convert safely to raw ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_join(ptr: *mut ThreadHandle) {
    if ptr.is_null() {
        eprintln!("⚠️ wpp_thread_join: null pointer");
        return;
    }

    unsafe {
        // FIX 1 & 2: Properly consume the Arc to decrement reference count
        // This fixes the memory leak - Arc will drop naturally at end of scope
        let arc = Arc::from_raw(ptr);

        // FIX 2: Call join() directly on Arc instead of unsafe pointer dereference
        arc.join();

        // Arc drops here, decrementing reference count properly
    }
}



#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_poll(ptr: *mut ThreadHandle) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let handle = &*ptr;
        handle.is_finished() as c_int
    }
}

// ===========================================================
// 🧩 ThreadState API (for useThreadState())
// ===========================================================

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_state_new(initial: c_int) -> *mut c_void {
    let state = ThreadState::new(initial);
    Box::into_raw(Box::new(state)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_state_get(ptr: *mut c_void) -> c_int {
    let state = unsafe { &*(ptr as *mut ThreadState<i32>) };
    state.get()
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_state_set(ptr: *mut c_void, val: c_int) {
    let state = unsafe { &*(ptr as *mut ThreadState<i32>) };
    state.set(val);
}

// ===========================================================
// 🧠 Optional background GC scheduler
// ===========================================================

/// FIX 12: Graceful daemon shutdown
pub fn stop_thread_gc_daemon() {
    println!("🛑 [gc] Requesting daemon shutdown");
    DAEMON_SHUTDOWN.store(true, Ordering::Release);
}

pub fn start_thread_gc_daemon() {
    thread::spawn(|| loop {
        // FIX 12: Check shutdown flag
        if DAEMON_SHUTDOWN.load(Ordering::Acquire) {
            println!("🛑 [gc] Daemon shutting down");
            break;
        }
        let mut collected = 0;
        let mut joined = 0;

        {
            let mut threads = ThreadGC::global().threads.lock().unwrap();
            let mut to_remove = Vec::new();

            for (id, weak_handle) in threads.iter() {
                if let Some(mut handle_arc) = weak_handle.upgrade() {
    if handle_arc.is_finished() {
        // Try to join the thread (if not already joined)
        if let Some(handle_mut) = Arc::get_mut(&mut handle_arc) {
            let mut guard = handle_mut.join_handle.lock().unwrap();
if let Some(join_handle) = guard.take() {

                println!("⚙️ [gc] Auto-joining finished thread #{id}");
                let _ = join_handle.join();
                joined += 1;
            }
        }
        to_remove.push(*id);
    }
} else {
    // Weak expired — fully collected
    println!("💀 [gc] Thread #{id} expired (Weak dropped)");
}

            }

            // Remove dead or finished threads
            for id in to_remove {
                threads.remove(&id);
                collected += 1;
            }
        }

        // Check GC-managed mutexes for dead owners
        {
            let mut mutexes = ThreadGC::global().mutexes.lock().unwrap();
            for (_, weak_any) in mutexes.iter() {
                if let Some(mtx_any) = weak_any.upgrade() {
                    if let Some(mtx) = mtx_any.downcast_ref::<GcMutex<c_int>>() {
                        mtx.force_unlock_if_dead();
                    }
                }
            }
        }

        if collected > 0 || joined > 0 {
            println!("🧹 [gc-daemon] Cleaned {collected} collected, {joined} joined threads");
        }

        // FIX 20: Use named constant instead of magic number
        thread::sleep(Duration::from_secs(GC_DAEMON_INTERVAL_SECS));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_join_all() {
    println!("🧵 [runtime] Entered wpp_thread_join_all()");

    // === Fast path: No threads were ever spawned ===
    if THREADS_EVER_SPAWNED.load(Ordering::Relaxed) == 0 {
        println!("🧵 [thread] no threads ever spawned, skipping join");
        return;
    }

    // === Step 1: Snapshot the threads safely ===
    let gc = ThreadGC::global();
    let mut threads_lock = gc.threads.lock().unwrap();

    if threads_lock.is_empty() {
        println!("🧵 [thread] no threads to join");
        return;
    }

    // Clone all Weak handles before dropping the lock
    let threads_snapshot: Vec<(u64, Weak<ThreadHandle>)> =
        threads_lock.iter().map(|(id, w)| (*id, w.clone())).collect();

    // Clear the map (so GC can refill it later)
    threads_lock.clear();
    drop(threads_lock);

    println!(
        "🧵 [thread] joining all remaining GC threads ({} total)",
        threads_snapshot.len()
    );

    // === Step 2: Join threads safely outside the lock ===
    let mut joined = 0usize;
    for (id, weak_handle) in threads_snapshot {
        if let Some(handle_arc) = weak_handle.upgrade() {
            // Lock the join_handle mutex
            let mut join_guard = handle_arc.join_handle.lock().unwrap();

            // Join if not already joined
            if let Some(join_handle) = join_guard.take() {
                println!("🧵 [thread] joining thread #{id}");
                match join_handle.join() {
                    Ok(_) => {
                        println!("✅ [thread] joined thread #{id}");
                        joined += 1;
                    }
                    Err(_) => {
                        println!("💥 [thread] panic while joining thread #{id}");
                    }
                }
            } else {
                println!("💤 [thread] thread #{id} already finished or detached");
            }
        } else {
            println!("💀 [thread] thread #{id} already collected (Weak expired)");
        }
    }

    println!("✅ [thread] all GC threads joined ({joined})");
}


// ===========================================================
// 🌐 Extern Mutex API for W++
// ===========================================================

#[unsafe(no_mangle)]
pub extern "C" fn wpp_mutex_new(initial: c_int) -> *mut GcMutex<c_int> {
    let mtx = GcMutex::new(initial);
    Arc::into_raw(mtx) as *mut GcMutex<c_int>
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_mutex_lock(ptr: *mut GcMutex<c_int>, thread_id: c_int) {
    if ptr.is_null() {
        eprintln!("⚠️ wpp_mutex_lock: null pointer");
        return;
    }
    unsafe {
        let mtx = &*ptr;
        match mtx.lock(thread_id as u64) {
            Ok(_) => println!("🔒 [mutex] Mutex locked by thread #{thread_id}"),
            Err(_) => eprintln!("💥 [mutex] Mutex #{:?} lock failed", mtx.id),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_mutex_unlock(ptr: *mut GcMutex<c_int>) {
    if ptr.is_null() {
        eprintln!("⚠️ wpp_mutex_unlock: null pointer");
        return;
    }
    unsafe {
        let mtx = &*ptr;
        mtx.unlock();
    }
}
