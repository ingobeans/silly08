use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::engine::{KEYS_AVAILABLE, Message, PRESSED_KEYS, RELEASED_KEYS, SCREEN_DATA};

mod engine;

fn window() -> Conf {
    Conf {
        window_title: "silly08".to_string(),
        window_width: 128 * 6,
        window_height: 128 * 6,
        ..Default::default()
    }
}

#[macroquad::main(window)]
async fn main() {
    let data = if let Some(path) = std::env::args().nth(1) {
        std::fs::read(path).unwrap()
    } else {
        include_bytes!("../test.p8").to_vec()
    };
    let (handle, rx, _) = engine::run_thread(data);

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
                Message::ScreenData => {
                    let data = SCREEN_DATA.lock().unwrap();
                    assert_eq!(data.len(), image.bytes.len());
                    image.bytes = data.clone();
                    texture.update(&image);
                }
            }
        }

        if handle.is_finished() {
            let err = handle.join().unwrap().unwrap_err();
            panic!("{err:?}");
        }

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

        let (mut screen_w, mut screen_h) = screen_size();
        let scale = screen_dpi_scale();
        screen_w /= scale;
        screen_h /= scale;
        let scale_factor = (screen_w / 128.0).min(screen_h / 128.0);
        let size = 128.0 * scale_factor;
        let x = (screen_w - size) / 2.0;
        let y = (screen_h - size) / 2.0;

        draw_texture_ex(
            &texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(size, size)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
