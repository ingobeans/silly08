use deno_core::extension;
use deno_core::op2;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::OnceLock;

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

extension!(my_ext, ops = [test, send_data]);

pub static MESSAGE_TX: OnceLock<std::sync::mpsc::Sender<Message>> = OnceLock::new();

#[derive(Debug)]
pub enum Message {
    Ping,
    Data(Vec<u8>),
}

async fn run() {
    let main_module = deno_core::url::Url::parse("node:edu.js").unwrap();
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![my_ext::init()],
        ..Default::default()
    });

    let base = std::fs::read_to_string("base.js").unwrap();
    let file = std::fs::read_to_string("edu.js").unwrap();
    let code = base + &file;

    let mod_id = js_runtime
        .load_main_es_module_from_code(&main_module, code)
        .await
        .unwrap();
    let result = js_runtime.mod_evaluate(mod_id);
    let _ = js_runtime.run_event_loop(Default::default()).await.unwrap();
    let _ = result.await.unwrap();
}

pub fn run_thread() -> (
    std::thread::JoinHandle<()>,
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
