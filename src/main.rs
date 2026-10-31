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

    let handle = run_thread(data);

    let texture = Texture2D::from_rgba8(128, 128, &[0; SCREEN_DATA_SIZE]);
    texture.set_filter(FilterMode::Nearest);

    let mut save_state: Vec<u8> = vec![0_u8; SAVE_STATE_SIZE];

    loop {
        if let Some(data) = SCREEN_DATA_MANAGER.get_new_frame() {
            assert_eq!(data.len(), SCREEN_DATA_SIZE);
            texture.update_from_bytes(128, 128, &data);
        }

        if handle.is_finished() {
            // print result of thread if it fails early
            let err = handle.join().unwrap();
            panic!("{err:?}");
        }

        if is_key_down(KeyCode::RightShift) {
            // While right shift is held, handle special events
            if is_key_pressed(KeyCode::S) {
                SAVE_STATE_MANAGER.get_save_blocking(save_state.as_mut_ptr());
                println!("saved state!");
            } else if is_key_pressed(KeyCode::A) {
                SAVE_STATE_MANAGER.load_save_blocking(save_state.as_ptr());
                println!("loadstate");
            }
        } else {
            // send regular key events
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
