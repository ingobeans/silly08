use base64::prelude::*;
use deno_core::{GarbageCollected, cppgc::GcCell, error::AnyError, extension, v8};
use std::{
    rc::Rc,
    sync::{Mutex, atomic::AtomicBool},
    thread::JoinHandle,
    time::Duration,
};

use crate::engine::js_glue::my_ext;
pub use crate::engine::types::*;

mod js_glue;
mod types;

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
pub static SAVE_STATE_MANAGER: SaveStateManager = SaveStateManager::new();
pub static SHOULD_TERMINATE: AtomicBool = AtomicBool::new(false);

pub static SCREEN_DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());

// Runtime:

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

pub fn run_thread(cart_data: Vec<u8>) -> JoinHandle<Result<(), AnyError>> {
    let handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(run(&cart_data))
    });

    handle
}
