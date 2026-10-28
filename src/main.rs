use macroquad::prelude::*;

mod engine;

#[macroquad::main("silly08")]
async fn main() {
    let (handle, rx) = engine::run_thread();
    loop {
        if let Ok(message) = rx.try_recv() {
            println!("{message:?}");
        }
        next_frame().await
    }
}
