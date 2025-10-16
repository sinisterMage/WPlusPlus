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
        });

        ThreadGC::register_mutex(mutex.clone());
        println!("🧩 [gc] Registered mutex #{id}");
        mutex
    }

    pub fn lock(&self, thread_id: u64) -> std::sync::LockResult<std::sync::MutexGuard<'_, T>> {
        if self.poisoned.load(Ordering::SeqCst) {
            eprintln!("⚠️ [mutex] Attempt to lock poisoned mutex #{:?}", self.id);
        }

        let guard = self.data.lock();
        match guard {
            Ok(g) => {
                *self.owner_thread.lock().unwrap() = Some(thread_id);
                Ok(g)
            }
            Err(poisoned) => {
                self.poisoned.store(true, Ordering::SeqCst);
                Err(poisoned)
            }
        }
    }

    pub fn unlock(&self) {
        *self.owner_thread.lock().unwrap() = None;
        println!("🔓 [mutex] Mutex #{} unlocked manually", self.id);
    }

    pub fn owner_dead(&self) -> bool {
    if let Some(owner) = *self.owner_thread.lock().unwrap() {
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
    let id = ptr.id;
    ThreadGC::global()
        .threads
        .lock()
        .unwrap()
        .insert(id, Arc::downgrade(&ptr));
    println!("🧠 [gc] Registered thread handle #{id}");
}

pub fn register_mutex<T: Send + 'static>(ptr: Arc<GcMutex<T>>) {
    let id = ptr.id;
    let erased: Arc<dyn Any + Send + Sync> = ptr;
    ThreadGC::global()
        .mutexes
        .lock()
        .unwrap()
        .insert(id, Arc::downgrade(&erased));
    println!("🧩 [gc] Registered mutex #{id}");
}


     pub fn collect_now() {
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

    for (_, weak_any) in mutexes.iter() {
        if let Some(mtx_any) = weak_any.upgrade() {
            if let Some(mtx) = mtx_any.downcast_ref::<GcMutex<c_int>>() {
                mtx.force_unlock_if_dead();
            }
        }
    }

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
            if let Some(join_handle) = handle_mut.join_handle.take() {
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

        // Run every few seconds
        thread::sleep(Duration::from_secs(3));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn wpp_thread_join_all() {
    let mut threads = ThreadGC::global().threads.lock().unwrap();
    if threads.is_empty() {
        println!("🧵 [thread] no threads to join");
        return;
    }

    println!(
        "🧵 [thread] joining all remaining GC threads ({} total)",
        threads.len()
    );

    let mut joined = 0;

    // Drain all Weak handles
    for (id, weak_handle) in threads.drain() {
        // Try to upgrade the Weak<ThreadHandle> to Arc<ThreadHandle>
        if let Some(mut handle_arc) = weak_handle.upgrade() {
            // Now we have an Arc<ThreadHandle> we can safely access
            let mut handle = Arc::get_mut(&mut handle_arc);
            if let Some(h) = handle {
                // Exclusive ownership: can take join handle directly
                if let Some(join_handle) = h.join_handle.take() {
                    println!("🧵 [thread] joining thread #{id}");
                    let _ = join_handle.join();
                    joined += 1;
                }
            } else {
                // Shared handle, just wait until finished
                if !handle_arc.is_finished() {
                    println!("💤 [thread] waiting on active thread #{id}");
                    let _ = handle_arc.join_handle.as_ref().map(|jh| jh.thread().unpark());
                }
            }
        } else {
            // Weak already expired — GC will handle it
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
