use lodepng::{Bitmap, RGBA};
use macroquad::prelude::*;

mod consts;
pub use consts::*;

/// Not to be confused with [`lodepng::Image`] or [`macroquad::Image`]
pub struct Image {
    palette: Vec<RGBA>,
    image: Bitmap<u8>,
}

impl Image {
    fn to_texture(&self) -> Texture2D {
        let Bitmap { width, height, .. } = self.image;
        let buffer = Vec::new();

        Texture2D::from_rgba8(
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            &buffer,
        )
    }
}

#[derive(Default)]
pub struct Editor {
    data: Image,
}

impl Editor {
    pub async fn run(&mut self) {
        loop {
            next_frame().await;
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            palette: vec![DEF_COLOR],
            image: Bitmap {
                width: DEF_SIZE,
                height: DEF_SIZE,
                buffer: vec![0; DEF_SIZE * DEF_SIZE],
            },
        }
    }
}
