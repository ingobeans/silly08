use base64::prelude::*;
use deno_core::GarbageCollected;
use deno_core::cppgc::GcCell;
use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::v8;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::engine::js_glue::my_ext;

mod js_glue;

// Constants:

/// The entire size of the heap memory of the runtime.
/// Found from testing, hardcoded :sob:
pub const MEMORY_SIZE: usize = 268435456;
/// The size of the memory saved in a save state. Arbitrary value.
/// From testing it seems that only about ~9% of the heap is used normally,
/// so for performance and storage reasons we only keep a smaller portion of the full memory.
/// (in this case 12.5%, giving a bit of headroom past the standard 9%)
pub const SAVED_MEMORY_SIZE: usize = MEMORY_SIZE / 8;
pub const SAVE_STATE_HEADER: &[u8] = b"silly08-00";
/// The total size of a save state, containing both the saved memory and the header.
pub const SAVE_STATE_SIZE: usize = SAVED_MEMORY_SIZE + SAVE_STATE_HEADER.len();

// Global state:

/// Active keystate
pub static KEY_STATE: KeyState = KeyState::new();

pub static SCREEN_DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());

pub static LOAD_STATE_AVAILABLE: AtomicBool = AtomicBool::new(false);
pub static SAVE_STATE_AVAILABLE: AtomicBool = AtomicBool::new(false);
pub static SAVE_STATE_DATA_PTR: Mutex<usize> = Mutex::new(0);

pub static MESSAGE_TX: Mutex<Option<std::sync::mpsc::Sender<Message>>> = Mutex::new(None);
pub static TERMINATE_RX: Mutex<Option<std::sync::mpsc::Receiver<StatusMessage>>> = Mutex::new(None);

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
    const fn new() -> Self {
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

async fn run(cart_data: &[u8]) -> Result<(), AnyError> {
    let main_module = deno_core::url::Url::parse("node:edu.js")?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![my_ext::init()],
        ..Default::default()
    });

    let cart = format!(
        "let sample = `data:application/pkcs8;base64,{}`;",
        BASE64_STANDARD.encode(cart_data)
    );
    let base = include_str!("../base.js").replacen("1", &SAVED_MEMORY_SIZE.to_string(), 1);
    let file = include_str!("../edu.js");
    let code = format!("{cart}\n{base}\n{file}");

    let mod_id = js_runtime
        .load_main_es_module_from_code(&main_module, code)
        .await?;
    let result = js_runtime.mod_evaluate(mod_id);

    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    Ok(())
}

pub fn run_thread(
    cart_data: Vec<u8>,
) -> (
    std::thread::JoinHandle<Result<(), AnyError>>,
    Receiver<Message>,
    Sender<StatusMessage>,
) {
    let (message_tx, message_rx) = channel();
    let (terminate_tx, terminate_rx) = channel();
    let handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        *MESSAGE_TX.lock().unwrap() = Some(message_tx);
        *TERMINATE_RX.lock().unwrap() = Some(terminate_rx);
        runtime.block_on(run(&cart_data))
    });

    (handle, message_rx, terminate_tx)
}
