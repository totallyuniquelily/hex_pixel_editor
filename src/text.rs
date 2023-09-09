// todo: don't allow(dead_code)
#![allow(dead_code)]
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
        texture.set_filter(FilterMode::Nearest);

        let texture_width = texture.width() as u64;
        let texture_height = texture.height() as u64;

        assert_eq!(texture_width, 48);
        assert_eq!(texture_height, 7);

        Ok(Self { texture })
    }

    /// draw a single hexadecimal digit. dimensions are 3*scale / 7*scale.
    pub fn draw_digit(&self, digit: u8, x: f32, y: f32, color: Color, scale: f32) {
        draw_texture_ex(
            &self.texture,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(3. * scale, 7. * scale)),
                source: Some(Rect::new(digit as f32 * 3., 0., 3., 7.)),
                ..Default::default()
            },
        );
    }
}
