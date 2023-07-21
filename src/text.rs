use anyhow::Result;
use macroquad::prelude::*;

pub struct FontImg {
    pub texture: Texture2D,
}

impl FontImg {
    /// load a font texture from path. wraps `load_texture`.
    /// asserts the texture hax valid dimensions
    pub async fn load(path: &str) -> Result<Self> {
        let texture = load_texture(path).await?;

        let texture_width = texture.width() as u64;
        let texture_height = texture.height() as u64;

        assert_eq!(texture_width, 48);
        assert_eq!(texture_height, 7);

        Ok(Self { texture })
    }

    pub fn draw_digit(digit: u8) {
        todo!();
    }
}
