use std::{
    any::Any,
    collections::HashMap,
    mem,
    os::raw::{c_int, c_void},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use once_cell::sync::Lazy;
use rand::Rng;

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
// 🧠 GC-Aware ThreadHandle
// ===========================================================
#[derive(Debug)]
pub struct ThreadHandle {
    pub id: u64,
    pub finished: Arc<AtomicBool>,
    pub result: Arc<Mutex<Option<Box<dyn Any + Send + 'static>>>>,
    pub join_handle: Option<thread::JoinHandle<()>>,
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
            join_handle: None,
            ref_count: Arc::new(AtomicU64::new(0)),
            
        });
        
    }

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

    let finished = Arc::new(AtomicBool::new(false));
    let result = Arc::new(Mutex::new(None));
    let ref_count = Arc::new(AtomicU64::new(1));

    let func: extern "C" fn() = unsafe { mem::transmute(func_ptr) };
    let fin_clone = finished.clone();

    println!("🚀 [thread] spawning GC-managed thread #{id}");

    let join_handle = Some(thread::spawn(move || {
    let result = std::panic::catch_unwind(|| func());
    match result {
        Ok(_) => println!("✅ [thread] thread #{id} finished normally"),
        Err(_) => eprintln!("💥 [thread] thread #{id} panicked"),
    }
    fin_clone.store(true, Ordering::SeqCst);
}));

    let handle = Arc::new(ThreadHandle {
        id,
        finished,
        result,
        join_handle,
        ref_count,
    });

    ThreadGC::register(handle.clone()); // ✅ register Arc safely
    handle
}


    pub fn join(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            println!("🧵 [thread] joining thread #{}", self.id);
            let _ = handle.join();
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
        let old = self.ref_count.fetch_sub(1, Ordering::SeqCst);
        if old == 1 {
            // last reference dropped
            println!("🧹 [gc] ThreadHandle #{} released (0 refs)", self.id);
            ThreadGC::collect_now();
        }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        if !self.is_finished() {
            println!("🧹 [thread] auto-joining unfinished thread #{}", self.id);
            self.join();
        }
    }
}

// ===========================================================
// 🧹 ThreadGC: Self-contained mark-and-sweep for threads
// ===========================================================
pub struct ThreadGC {
    threads: Mutex<HashMap<u64, Arc<ThreadHandle>>>,
}

impl ThreadGC {
    fn global() -> &'static ThreadGC {
        static INSTANCE: Lazy<ThreadGC> = Lazy::new(|| ThreadGC {
            threads: Mutex::new(HashMap::new()),
        });
        &INSTANCE
    }

    pub fn register(ptr: Arc<ThreadHandle>) {
        let id = ptr.id;
        ThreadGC::global()
            .threads
            .lock()
            .unwrap()
            .insert(id, ptr.clone());
        println!("🧠 [gc] Registered thread handle #{id}");
    }

    pub fn collect_now() {
        let mut threads = ThreadGC::global().threads.lock().unwrap();
        let mut collected = 0;

        threads.retain(|id, handle| {
            if handle.is_finished() && Arc::strong_count(handle) == 1 {
                println!("💀 [gc] Collecting thread #{id}");
                collected += 1;
                false
            } else {
                true
            }
        });

        if collected > 0 {
            println!("🧹 [gc] Collected {collected} finished threads");
        }
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
        // temporarily clone the Arc without consuming the original pointer
        let arc_ref = Arc::from_raw(ptr);
        let cloned = arc_ref.clone();
        std::mem::forget(arc_ref); // leave the original raw Arc alive

        // safely join
        let mut_ref = Arc::as_ptr(&cloned) as *mut ThreadHandle;
        (*mut_ref).join();

        // dropping `cloned` here decrements ref count correctly
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
pub fn start_thread_gc_daemon() {
    thread::spawn(|| loop {
        ThreadGC::collect_now();
        thread::sleep(Duration::from_secs(3));
    });
}
