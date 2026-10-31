use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};

use crate::engine::*;

struct SaveStateManagerInner {
    dest_ptr: *mut u8,
    requested_save: bool,
}
unsafe impl Send for SaveStateManagerInner {}

impl SaveStateManagerInner {
    const fn new() -> Self {
        Self {
            dest_ptr: std::ptr::null_mut(),
            requested_save: false,
        }
    }
}

pub struct SaveStateManager {
    inner: Mutex<SaveStateManagerInner>,
    save_state_available: AtomicBool,
}
impl SaveStateManager {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(SaveStateManagerInner::new()),
            save_state_available: AtomicBool::new(false),
        }
    }
    /// Writes a save state from memory data.
    /// Note: memory_data should not contain the header.
    pub fn write_save(&self, memory_data: &[u8]) {
        let lock = self.inner.lock().unwrap();

        if lock.dest_ptr.is_null() {
            panic!("SAVE_STATE_DATA_PTR is null!");
        }

        unsafe {
            // Write header
            std::ptr::copy(
                SAVE_STATE_HEADER.as_ptr(),
                lock.dest_ptr,
                SAVE_STATE_HEADER.len(),
            );

            // Write memory data
            std::ptr::copy(
                memory_data.as_ptr(),
                lock.dest_ptr.add(SAVE_STATE_HEADER.len()),
                memory_data.len().min(SAVED_MEMORY_SIZE),
            );
        }

        self.save_state_available.store(true, Ordering::Relaxed);
    }
    pub fn get_save(&self, dest_ptr: *mut u8) {
        let mut lock = self.inner.lock().unwrap();
        assert!(!lock.requested_save);

        self.save_state_available.store(false, Ordering::Relaxed);
        lock.dest_ptr = dest_ptr;
    }
    pub fn get_save_blocking(&self, dest_ptr: *mut u8) {
        self.get_save(dest_ptr);
        // wait for write
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

#[allow(dead_code)]
#[derive(Debug)]
pub enum StatusMessage {
    Terminate,
    SaveState,
    LoadState(usize),
}

#[derive(Debug)]
pub enum Message {
    ScreenData,
}
