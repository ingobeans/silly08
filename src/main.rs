use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::engine::{HELD_KEYS, Message, PRESSED_KEYS};

mod engine;

#[macroquad::main("silly08")]
async fn main() {
    let (_handle, rx) = engine::run_thread();

    let image_data: [u8; 128 * 128 * 4] = std::array::from_fn(|_| 0);
    let mut image = Image {
        bytes: image_data.to_vec(),
        width: 128,
        height: 128,
    };
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    loop {
        if let Ok(message) = rx.try_recv() {
            match message {
                Message::Data(data) => {
                    assert_eq!(data.len(), image.bytes.len());
                    image.bytes = data;
                    texture.update(&image);
                }
                _ => {
                    println!("{message:?}");
                }
            }
        }
        let mut held = Vec::new();
        for key in get_keys_down() {
            let mut key = format!("{:?}", key);
            if key.len() == 1 {
                key = key.to_lowercase();
            }
            held.push(key);
        }
        *HELD_KEYS.lock().unwrap() = held;

        let mut pressed = Vec::new();
        for key in get_keys_pressed() {
            let mut key = format!("{:?}", key);
            if key.len() == 1 {
                key = key.to_lowercase();
            }
            pressed.push(key);
        }
        *PRESSED_KEYS.lock().unwrap() = pressed;

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
