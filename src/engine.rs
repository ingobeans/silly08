use base64::prelude::*;
use deno_core::GarbageCollected;
use deno_core::cppgc::GcCell;
use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::op2;
use deno_core::v8;
use deno_core::v8::ValueDeserializerHelper;
use deno_core::v8::ValueSerializerHelper;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::time::Duration;

// from testing, hardcoded :sob:
pub const MEMORY_SIZE: usize = 268435456;

struct DefaultSerializer;

impl v8::ValueSerializerImpl for DefaultSerializer {
    fn throw_data_clone_error<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        message: v8::Local<'s, v8::String>,
    ) {
        eprintln!("Error serializing::{}", message.to_rust_string_lossy(scope));
    }
}
impl v8::ValueDeserializerImpl for DefaultSerializer {}

#[op2]
fn serialize(value: v8::Local<'_, v8::Value>, scope: &v8::PinScope<'_, '_>) -> Vec<u8> {
    let context = scope.get_current_context();
    let serializer = v8::ValueSerializer::new(scope, Box::new(DefaultSerializer));
    serializer.write_header();
    serializer.write_value(context, value);
    serializer.release()
}

#[op2]
fn deserialize<'t>(
    #[buffer] data: &[u8],
    scope: &v8::PinScope<'t, '_>,
) -> Option<v8::Local<'t, v8::Value>> {
    let context = scope.get_current_context();

    let serializer = v8::ValueDeserializer::new(scope, Box::new(DefaultSerializer), data);
    serializer.read_header(context);
    serializer.read_value(context)
}
#[op2(fast)]
fn send_save_state(#[buffer(copy)] data: Vec<u8>) {
    let ptr = SAVE_STATE_DATA_PTR.lock().unwrap();
    if *ptr == 0 {
        eprintln!("SAVE_STATE_DATA_PTR is null!");
        return;
    }
    let ptr = *ptr as *mut u8;

    unsafe {
        std::ptr::copy(data.as_ptr(), ptr, data.len());
    }
    SAVE_STATE_AVAILABLE.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[op2(fast)]
fn send_data(#[buffer(copy)] data: Vec<u8>) {
    *SCREEN_DATA.lock().unwrap() = data;
    MESSAGE_TX
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .send(Message::ScreenData)
        .unwrap();
}

#[repr(C)]
pub struct MessagePollObject {
    keys_available: GcCell<bool>,
    should_terminate: GcCell<bool>,
    should_save_state: GcCell<bool>,
    should_load_state: GcCell<bool>,
    pressed_keys: GcCell<Vec<String>>,
    released_keys: GcCell<Vec<String>>,
}

#[op2]
impl MessagePollObject {
    #[getter]
    fn keys_available(&self, isolate: &mut v8::Isolate) -> bool {
        let a = *self.keys_available.get(isolate);
        self.keys_available.set(isolate, false);
        a
    }
    #[getter]
    fn should_terminate(&self, isolate: &v8::Isolate) -> bool {
        *self.should_terminate.get(isolate)
    }
    #[getter]
    fn should_save_state(&self, isolate: &v8::Isolate) -> bool {
        *self.should_save_state.get(isolate)
    }
    #[getter]
    fn should_load_state(&self, isolate: &v8::Isolate) -> bool {
        *self.should_load_state.get(isolate)
    }
    #[getter]
    fn pressed_keys(&self, isolate: &v8::Isolate) -> Vec<String> {
        self.pressed_keys.get(isolate).clone()
    }
    #[getter]
    fn released_keys(&self, isolate: &v8::Isolate) -> Vec<String> {
        self.released_keys.get(isolate).clone()
    }

    #[static_method]
    #[cppgc]
    fn poll(#[buffer] load_state: &mut [u8]) -> MessagePollObject {
        let mut should_terminate = false;
        let mut should_save_state = false;
        let mut should_load_state = false;

        // let mut load_state = Vec::new();

        if let Ok(message) = TERMINATE_RX.lock().unwrap().as_mut().unwrap().try_recv() {
            match message {
                StatusMessage::Terminate => should_terminate = true,
                StatusMessage::SaveState => should_save_state = true,
                StatusMessage::LoadState(ptr) => {
                    should_load_state = true;
                    let ptr = ptr as *mut u8;

                    unsafe {
                        std::ptr::copy(ptr, load_state.as_mut_ptr(), load_state.len());
                    }
                    LOAD_STATE_AVAILABLE.store(false, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }

        let mut lock = KEYS_AVAILABLE.lock().unwrap();

        let keys_available = *lock;
        let mut pressed_keys = Vec::new();
        let mut released_keys = Vec::new();

        if *lock {
            *lock = false;
            pressed_keys = PRESSED_KEYS.lock().unwrap().drain(..).collect();
            released_keys = RELEASED_KEYS.lock().unwrap().drain(..).collect();
        }
        MessagePollObject {
            should_terminate: GcCell::new(should_terminate),
            should_save_state: GcCell::new(should_save_state),
            keys_available: GcCell::new(keys_available),
            pressed_keys: GcCell::new(pressed_keys),
            released_keys: GcCell::new(released_keys),
            should_load_state: GcCell::new(should_load_state),
        }
    }
}

unsafe impl GarbageCollected for MessagePollObject {
    fn trace(&self, _visitor: &mut v8::cppgc::Visitor) {}
    fn get_name(&self) -> &'static std::ffi::CStr {
        c"MessagePollObject"
    }
}

#[op2(async(lazy), fast)]
async fn timeout(#[bigint] time: u64) {
    tokio::time::sleep(Duration::from_millis(time)).await;
}

extension!(
    my_ext,
    ops = [send_data, timeout, serialize, deserialize, send_save_state],
    objects = [MessagePollObject],
);

pub static KEYS_AVAILABLE: Mutex<bool> = Mutex::new(false);
pub static RELEASED_KEYS: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static PRESSED_KEYS: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static SCREEN_DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());

pub static LOAD_STATE_AVAILABLE: AtomicBool = AtomicBool::new(false);
pub static SAVE_STATE_AVAILABLE: AtomicBool = AtomicBool::new(false);
pub static SAVE_STATE_DATA_PTR: Mutex<usize> = Mutex::new(0);

pub static MESSAGE_TX: Mutex<Option<std::sync::mpsc::Sender<Message>>> = Mutex::new(None);
pub static TERMINATE_RX: Mutex<Option<std::sync::mpsc::Receiver<StatusMessage>>> = Mutex::new(None);

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
    let base = include_str!("../base.js");
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
