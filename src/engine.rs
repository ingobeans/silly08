use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::op2;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;

#[op2(fast)]
fn test() -> f64 {
    println!("EHBGEF");
    MESSAGE_TX.get().unwrap().send(Message::Ping).unwrap();
    4.5
}
#[op2(fast)]
fn send_data(#[buffer(copy)] data: Vec<u8>) {
    MESSAGE_TX.get().unwrap().send(Message::Data(data)).unwrap();
}
#[op2(async(lazy), fast)]
async fn timeout(#[bigint] time: u64) -> (Vec<String>, Vec<String>) {
    tokio::time::sleep(Duration::from_millis(time)).await;
    let mut lock = BOOTED.lock().unwrap();
    if !*lock {
        *lock = true;
        MESSAGE_TX.get().unwrap().send(Message::Booted).unwrap();
    }
    (
        PRESSED_KEYS.lock().unwrap().clone(),
        HELD_KEYS.lock().unwrap().clone(),
    )
}

extension!(my_ext, ops = [test, send_data, timeout]);

pub static PRESSED_KEYS: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static HELD_KEYS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub static BOOTED: Mutex<bool> = Mutex::new(false);
pub static MESSAGE_TX: OnceLock<std::sync::mpsc::Sender<Message>> = OnceLock::new();

#[derive(Debug)]
pub enum Message {
    Ping,
    Booted,
    Data(Vec<u8>),
}

async fn run() -> Result<(), AnyError> {
    let main_module = deno_core::url::Url::parse("node:edu.js")?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![my_ext::init()],
        ..Default::default()
    });

    let base = std::fs::read_to_string("base.js")?;
    let file = std::fs::read_to_string("edu.js")?;
    let code = base + &file;

    let a = js_runtime
        .load_side_es_module_from_code(
            &deno_core::ModuleSpecifier::parse("fake-indexeddb:auto")?,
            "class DOMException {}".to_string() + include_str!("../fake-indexeddb.js"),
        )
        .await?;
    let lib_result = js_runtime.mod_evaluate(a);

    let mod_id = js_runtime
        .load_main_es_module_from_code(&main_module, code)
        .await?;
    let result = js_runtime.mod_evaluate(mod_id);

    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    lib_result.await?;
    Ok(())
}

pub fn run_thread() -> (
    std::thread::JoinHandle<Result<(), AnyError>>,
    std::sync::mpsc::Receiver<Message>,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        MESSAGE_TX.set(tx).unwrap();
        runtime.block_on(run())
    });

    (handle, rx)
}
