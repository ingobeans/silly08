use base64::prelude::*;
use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::op2;
use deno_core::v8;
use deno_core::v8::ValueDeserializerHelper;
use deno_core::v8::ValueSerializerHelper;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;

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
fn send_data(#[buffer(copy)] data: Vec<u8>) {
    MESSAGE_TX
        .get()
        .unwrap()
        .send(Message::ScreenData(data))
        .unwrap();
}
#[op2(async(lazy), fast)]
async fn timeout(#[bigint] time: u64) -> (bool, Vec<String>, Vec<String>) {
    tokio::time::sleep(Duration::from_millis(time)).await;

    if time != 0 {
        let mut lock = KEYS_AVAILABLE.lock().unwrap();
        if *lock {
            *lock = false;
            return (
                true,
                PRESSED_KEYS.lock().unwrap().drain(..).collect(),
                RELEASED_KEYS.lock().unwrap().drain(..).collect(),
            );
        }
    }
    (false, Vec::new(), Vec::new())
}

extension!(my_ext, ops = [send_data, timeout, serialize, deserialize]);

pub static KEYS_AVAILABLE: Mutex<bool> = Mutex::new(false);
pub static RELEASED_KEYS: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static PRESSED_KEYS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub static MESSAGE_TX: OnceLock<std::sync::mpsc::Sender<Message>> = OnceLock::new();

#[derive(Debug)]
pub enum Message {
    ScreenData(Vec<u8>),
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
    let base = std::fs::read_to_string("base.js")?;
    let file = std::fs::read_to_string("edu.js")?;
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
    std::sync::mpsc::Receiver<Message>,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        MESSAGE_TX.set(tx).unwrap();
        runtime.block_on(run(&cart_data))
    });

    (handle, rx)
}
