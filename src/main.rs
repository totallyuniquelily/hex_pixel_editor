use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

mod image;

const BG_COLOR: Color = color_u8!(60, 10, 0, 255);

#[macroquad::main("Png editor")]
async fn main() -> Result<()> {
    let mut img = {
        // Parse arg
        let mut args = std::env::args();
        if args.len() != 2 {
            println!("Usage: {} filename", args.next().unwrap());
            return Ok(());
        }
        let arg = args.nth(1).unwrap();
        let path = Path::new(&arg);
        image::Image::from_path(path)?
    };

    let mut img_scale = 1.0;
    // palette panel size
    let mut panel = PanelSettings {
        padding: 3.0,
        inner_width: 208.0,
        visible: true,
    };
    /// placeholder, whether currently asking for palette indexes to swap
    let mut prompt = false;
    let mut promptbuf = String::new();
    loop {
        clear_background(BG_COLOR);

        if !prompt {
            // Handle input
            // Scale buttons
            if is_key_pressed(KeyCode::Equal) {
                img_scale += 0.1;
            }
            if is_key_pressed(KeyCode::Minus) {
                img_scale -= 0.1;
            }
            if is_key_pressed(KeyCode::P) {
                panel.visible = !panel.visible;
            }

            if is_key_pressed(KeyCode::T) {
                prompt = true;
                // clear char buffer just in case
                while get_char_pressed().is_some() {}
            }
        }
        if prompt && is_key_pressed(KeyCode::Enter) {
            get_char_pressed();
            // implement swap here ig
            promptbuf.clear();
            prompt = false;
        }

        // Draw Image and panel
        panel.draw_image(img.texture(), img_scale);
        panel.draw_panel(&img);

        if prompt {
            const text: &str = "Palette indexes to swap (separate with space):";
            if let Some(ch) = get_char_pressed() {
                if !ch.is_control() {
                    promptbuf.push(ch);
                    println!("adding {:?}", ch);
                }
            }
            let dim1 = measure_text(text, None, 20, 1.0);
            let dim2 = measure_text(&promptbuf, None, 20, 1.0);
            draw_rectangle(
                0.0,
                screen_height() - dim1.height * 1.5,
                dim1.width + dim2.width,
                screen_height(),
                BLACK,
            );
            draw_text_ex(
                text,
                0.0,
                screen_height() - dim1.height / 2.,
                TextParams::default(),
            );
            draw_text_ex(
                &promptbuf,
                dim1.width,
                screen_height() - dim2.height / 2.,
                TextParams::default(),
            );
        }

        next_frame().await;
    }
}

struct PanelSettings {
    padding: f32,
    inner_width: f32,
    // panel_width: f32,
    visible: bool,
}

impl PanelSettings {
    #[inline]
    fn panel_width(&self) -> f32 {
        if self.visible {
            self.inner_width + self.padding
        } else {
            0.0
        }
    }

    fn draw_image(&self, texture: &Texture2D, img_scale: f32) {
        let aspect_ratio = texture.width() / texture.height();
        let s =
            (screen_height() * aspect_ratio).min(screen_width() - self.panel_width()) * img_scale;
        draw_texture_ex(
            texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(s, s / aspect_ratio)),
                ..Default::default()
            },
        );
    }

    fn draw_panel(&self, img: &image::Image) {
        if !self.visible {
            return;
        }
        let topx = screen_width() - self.panel_width();
        draw_rectangle(topx, 0.0, self.panel_width(), screen_height(), BLACK);
        for (i, rgb) in img.palette().iter().enumerate() {
            let i = i as u8;
            let logical_x = (i % 16) as f32;
            let logical_y = (i / 16) as f32;
            let x = logical_x * self.inner_width / 16. + topx;
            let y = logical_y * self.inner_width / 16.;
            let color = Color::from_rgba(rgb.r, rgb.g, rgb.b, 255);
            draw_rectangle(
                x + self.padding,
                y + self.padding,
                self.inner_width / 16. - self.padding,
                self.inner_width / 16. - self.padding,
                color,
            );
        }
    }
}
