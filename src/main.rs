use std::rc::Rc;

async fn run_js() {
    let main_module = deno_core::resolve_path("edu.js", &std::env::current_dir().unwrap()).unwrap();
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
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
    let _ = js_runtime.run_event_loop(Default::default()).await;
    let _ = result.await;
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(run_js())
}
