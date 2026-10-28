use macroquad::prelude::*;

use crate::engine::Message;

mod engine;

#[macroquad::main("silly08")]
async fn main() {
    let (handle, rx) = engine::run_thread();

    let image_data: [u8; 128 * 128 * 4] = std::array::from_fn(|_| 0);
    let mut image = Image {
        bytes: image_data.to_vec(),
        width: 128,
        height: 128,
    };
    let texture = Texture2D::from_image(&image);

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
        draw_texture(&texture, 0.0, 0.0, WHITE);
        next_frame().await
    }
}
