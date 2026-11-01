use base64::prelude::*;
use deno_core::{GarbageCollected, cppgc::GcCell, error::AnyError, extension, v8};
use std::{rc::Rc, sync::atomic::AtomicBool, thread::JoinHandle, time::Duration};

use crate::engine::js_glue::glue_ext;
pub use crate::engine::{consts::*, types::*};

mod consts;
mod js_glue;
mod types;

// Global state:

pub static KEY_STATE: KeyState = KeyState::new();
pub static SAVE_STATE_MANAGER: SaveStateManager = SaveStateManager::new();
pub static SHOULD_TERMINATE: AtomicBool = AtomicBool::new(false);
pub static SCREEN_DATA_MANAGER: ScreenDataManager = ScreenDataManager::new();

// Runtime:

async fn run(cart_data: &[u8]) -> Result<(), AnyError> {
    SHOULD_TERMINATE.store(false, std::sync::atomic::Ordering::Relaxed);
    let main_module = deno_core::url::Url::parse("node:edu.js")?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![glue_ext::init()],
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

pub fn run_thread(cart_data: Vec<u8>) -> JoinHandle<Result<(), AnyError>> {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(run(&cart_data))
    })
}
