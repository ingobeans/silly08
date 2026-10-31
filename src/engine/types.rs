use std::sync::{Mutex, MutexGuard};

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
