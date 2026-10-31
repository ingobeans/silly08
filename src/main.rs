use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::engine::*;

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
    let (handle, message_rx, sender_tx) = run_thread(data);

    let image_data: [u8; 128 * 128 * 4] = std::array::from_fn(|_| 0);
    let mut image = Image {
        bytes: image_data.to_vec(),
        width: 128,
        height: 128,
    };
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let mut save_state: Vec<u8> = vec![0_u8; SAVE_STATE_SIZE];

    loop {
        if let Ok(message) = message_rx.try_recv() {
            match message {
                Message::ScreenData => {
                    let data = SCREEN_DATA.lock().unwrap();
                    assert_eq!(data.len(), image.bytes.len());
                    image.bytes = data.clone();
                    texture.update(&image);
                }
            }
        }

        if is_key_down(KeyCode::RightShift) {
            if is_key_pressed(KeyCode::S) {
                sender_tx.send(StatusMessage::SaveState).unwrap();
                SAVE_STATE_MANAGER.get_save_blocking(save_state.as_mut_ptr());

                println!("saved state!");
            } else if is_key_pressed(KeyCode::A) {
                sender_tx
                    .send(StatusMessage::LoadState(save_state.as_mut_ptr() as usize))
                    .unwrap();
                println!("loadstate");
            }
        } else {
            if handle.is_finished() {
                let err = handle.join().unwrap().unwrap_err();
                panic!("{err:?}");
            }
            let mut state = KEY_STATE.update_lock();

            for key in get_keys_pressed() {
                state.pressed_keys.push(format!("{key:?}"));
            }

            for key in get_keys_released() {
                state.released_keys.push(format!("{key:?}"));
            }
        }

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
