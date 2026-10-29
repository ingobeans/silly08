use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::engine::{HELD_KEYS, KEYS_AVAILABLE, Message, PRESSED_KEYS, RELEASED_KEYS};

mod engine;

#[macroquad::main("silly08")]
async fn main() {
    let data = if let Some(path) = std::env::args().nth(1) {
        std::fs::read(path).unwrap()
    } else {
        include_bytes!("../test.p8").to_vec()
    };
    let (handle, rx) = engine::run_thread(data);

    let image_data: [u8; 128 * 128 * 4] = std::array::from_fn(|_| 0);
    let mut image = Image {
        bytes: image_data.to_vec(),
        width: 128,
        height: 128,
    };
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let mut booted = false;
    loop {
        if let Ok(message) = rx.try_recv() {
            match message {
                Message::Data(data) => {
                    assert_eq!(data.len(), image.bytes.len());
                    image.bytes = data;
                    texture.update(&image);
                }
                Message::Booted => booted = true,
                _ => {
                    println!("{message:?}");
                }
            }
        }

        if handle.is_finished() {
            let err = handle.join().unwrap().unwrap_err();
            panic!("{err:?}");
        }

        let mut held = Vec::new();
        for key in get_keys_down() {
            held.push(format!("{key:?}"));
        }
        HELD_KEYS.lock().unwrap().append(&mut held);

        let mut pressed = Vec::new();
        for key in get_keys_pressed() {
            pressed.push(format!("{key:?}"));
        }
        PRESSED_KEYS.lock().unwrap().append(&mut pressed);

        let mut released = Vec::new();
        for key in get_keys_released() {
            released.push(format!("{key:?}"));
        }
        RELEASED_KEYS.lock().unwrap().append(&mut released);

        *KEYS_AVAILABLE.lock().unwrap() = true;

        if booted {
            booted = false;
            let mut keys: Vec<String> = "run".chars().map(|a| a.to_string()).collect();
            keys.push("Enter".to_string());
            *PRESSED_KEYS.lock().unwrap() = keys;
        }

        let (mut w, mut h) = screen_size();
        let scale = screen_dpi_scale();
        w /= scale;
        h /= scale;
        let scale_factor = (w / 128.0).min(h / 128.0);

        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(128.0 * scale_factor, 128.0 * scale_factor)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
