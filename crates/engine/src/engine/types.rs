use std::sync::{
    Mutex, MutexGuard, RwLock,
    atomic::{AtomicBool, Ordering},
};

use crate::engine::*;

struct ScreenDataManagerInner {
    pub ptr: *const u8,
}
unsafe impl Send for ScreenDataManagerInner {}
unsafe impl Sync for ScreenDataManagerInner {}

pub struct ScreenDataManager {
    inner: RwLock<ScreenDataManagerInner>,
    frame_available: AtomicBool,
    ptr_initialized: AtomicBool,
}
impl ScreenDataManager {
    pub const fn new() -> Self {
        Self {
            inner: RwLock::new(ScreenDataManagerInner {
                ptr: std::ptr::null(),
            }),
            frame_available: AtomicBool::new(false),
            ptr_initialized: AtomicBool::new(false),
        }
    }
    #[allow(dead_code)]
    pub fn reset(&self) {
        self.frame_available.store(false, Ordering::Relaxed);
        self.ptr_initialized.store(false, Ordering::Relaxed);
    }
    pub fn get_new_frame(&self) -> Option<*const u8> {
        if self.frame_available.swap(false, Ordering::Relaxed) {
            return Some(self.inner.read().unwrap().ptr);
        }
        None
    }
    pub fn update_frame(&self, ptr: *const u8) {
        if SHOULD_TERMINATE.load(Ordering::Relaxed) {
            return;
        }
        if !self.ptr_initialized.swap(true, Ordering::Relaxed) {
            self.inner.write().unwrap().ptr = ptr;
        }
        self.frame_available.store(true, Ordering::Relaxed);
    }
}

struct SaveCtx {
    dest_ptr: *mut u8,
    requested_save: bool,
}
unsafe impl Send for SaveCtx {}

impl SaveCtx {
    const fn new() -> Self {
        Self {
            dest_ptr: std::ptr::null_mut(),
            requested_save: false,
        }
    }
}
struct LoadCtx {
    src_ptr: *const u8,
    requested_load: bool,
}
unsafe impl Send for LoadCtx {}

impl LoadCtx {
    const fn new() -> Self {
        Self {
            src_ptr: std::ptr::null_mut(),
            requested_load: false,
        }
    }
}

pub struct SaveStateManager {
    save_inner: Mutex<SaveCtx>,
    load_inner: Mutex<LoadCtx>,
    save_state_available: AtomicBool,
    load_state_complete: AtomicBool,
}
impl SaveStateManager {
    pub const fn new() -> Self {
        Self {
            save_inner: Mutex::new(SaveCtx::new()),
            load_inner: Mutex::new(LoadCtx::new()),
            save_state_available: AtomicBool::new(false),
            load_state_complete: AtomicBool::new(false),
        }
    }

    pub fn is_save_requested(&self) -> bool {
        self.save_inner.lock().unwrap().requested_save
    }

    /// Writes the internal state save to a destination memory buffer.
    /// Note: Only the memory part of the internal save is written.
    /// Intended for internal use.
    pub fn load_internal_save_if_requested(&self, dest: &mut [u8]) {
        let mut lock = self.load_inner.lock().unwrap();

        if !lock.requested_load {
            return;
        }
        lock.requested_load = false;

        if lock.src_ptr.is_null() {
            panic!("lock.src_ptr.is_null()");
        }

        unsafe {
            // Get header
            let mut header = [0u8; SAVE_STATE_HEADER.len()];
            std::ptr::copy_nonoverlapping(
                lock.src_ptr,
                header.as_mut_ptr(),
                SAVE_STATE_HEADER.len(),
            );

            if header != SAVE_STATE_HEADER {
                eprintln!(
                    "Unknown save state header encountered: {}\n({header:?})",
                    String::from_utf8_lossy(&header)
                );
            } else {
                // Get memory data
                std::ptr::copy_nonoverlapping(
                    lock.src_ptr.add(SAVE_STATE_HEADER.len()),
                    dest.as_mut_ptr(),
                    SAVED_MEMORY_SIZE,
                );
            }
        }

        self.load_state_complete.store(true, Ordering::Relaxed);
    }

    /// Updates internal state by writing a save state from memory data.
    /// Note: memory_data should not contain the header.
    /// Intended for internal use.
    pub fn write_internal_save(&self, memory_data: &[u8]) {
        let mut lock = self.save_inner.lock().unwrap();

        assert!(lock.requested_save);
        lock.requested_save = false;

        if lock.dest_ptr.is_null() {
            panic!("lock.dest_ptr.is_null()");
        }

        unsafe {
            // Write header
            std::ptr::copy_nonoverlapping(
                SAVE_STATE_HEADER.as_ptr(),
                lock.dest_ptr,
                SAVE_STATE_HEADER.len(),
            );

            // Write memory data
            std::ptr::copy_nonoverlapping(
                memory_data.as_ptr(),
                lock.dest_ptr.add(SAVE_STATE_HEADER.len()),
                memory_data.len().min(SAVED_MEMORY_SIZE),
            );
        }

        self.save_state_available.store(true, Ordering::Relaxed);
    }

    pub fn load_save(&self, src_ptr: *const u8) {
        let mut lock = self.load_inner.lock().unwrap();
        assert!(!lock.requested_load);
        lock.requested_load = true;

        self.load_state_complete.store(false, Ordering::Relaxed);
        lock.src_ptr = src_ptr;
    }
    pub fn load_save_blocking(&self, src_ptr: *const u8) {
        self.load_save(src_ptr);
        // Wait for completion
        while !self.load_state_complete.load(Ordering::Relaxed) {
            std::hint::spin_loop();
        }
    }

    pub fn get_save(&self, dest_ptr: *mut u8) {
        let mut lock = self.save_inner.lock().unwrap();
        assert!(!lock.requested_save);
        lock.requested_save = true;

        self.save_state_available.store(false, Ordering::Relaxed);
        lock.dest_ptr = dest_ptr;
    }
    pub fn get_save_blocking(&self, dest_ptr: *mut u8) {
        self.get_save(dest_ptr);
        // Wait for write
        while !self.save_state_available.load(Ordering::Relaxed) {
            std::hint::spin_loop();
        }
    }
}

pub struct KeyStateInner {
    pub pressed_keys: Vec<String>,
    pub released_keys: Vec<String>,
    pub new_key_events: bool,
}
impl KeyStateInner {
    const fn new() -> Self {
        Self {
            pressed_keys: Vec::new(),
            released_keys: Vec::new(),
            new_key_events: false,
        }
    }
}
pub struct KeyState {
    inner: Mutex<KeyStateInner>,
}
impl KeyState {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(KeyStateInner::new()),
        }
    }
    pub fn update_lock(&self) -> MutexGuard<'_, KeyStateInner> {
        let mut lock = self.inner.lock().unwrap();
        lock.new_key_events = true;
        lock
    }
    pub fn flush_events(&self) -> KeyStateInner {
        let mut lock = self.inner.lock().unwrap();
        let mut flushed = KeyStateInner::new();
        if lock.new_key_events {
            std::mem::swap(&mut flushed, &mut lock);
            lock.new_key_events = false;
        }
        flushed
    }
}
