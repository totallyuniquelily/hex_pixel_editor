use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;
use png::BitDepth;
use rgb::RGB8 as RGB;

mod image;

const BG_COLOR: Color = color_u8!(60, 10, 0, 255);

#[macroquad::main("Png editor")]
async fn main() -> Result<()> {
    // Parse arg
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Usage: {} filename", args.next().unwrap());
        return Ok(());
    }
    let arg = args.nth(1).unwrap();
    // Open file
    let path = Path::new(&arg);
    let file = std::fs::File::open(path);
    let Ok(file) = file else {
        let e = file.unwrap_err();
        panic!("error while opening file: {e}");
    };

    // Decode png
    let png = png::Decoder::new(file);
    let mut png = png.read_info()?;

    assert!(
        png.info().color_type == png::ColorType::Indexed,
        "Image must be palette-based"
    );

    let mut palette = Vec::with_capacity(256);
    match png.info().palette.clone() {
        Some(plte) => {
            for i in (0..plte.len()).step_by(3) {
                palette.push(RGB::new(plte[i], plte[i + 1], plte[i + 2]));
            }
        }
        None => {
            palette.extend([RGB::default(); 2]);
        }
    }

    let trns = match png.info().trns.clone() {
        Some(c) => c.into_owned(),
        None => Vec::with_capacity(255),
    };

    let mut buf = vec![0; png.output_buffer_size()];
    let o_info = png.next_frame(&mut buf)?;
    let buf = match o_info.bit_depth {
        BitDepth::Sixteen => panic!("16-bit colormap png!?"),
        BitDepth::Eight => buf,
        bitdepth => image::unpack(&buf, bitdepth, o_info.width as usize),
    };

    let img = image::Image::from_buffers(o_info.width, o_info.height, buf, palette, trns);

    let texture = img.to_texture();
    let mut img_scale = 1.0;
    loop {
        clear_background(BG_COLOR);

        // Handle input
        // Scale buttons
        if is_key_pressed(KeyCode::Equal) {
            img_scale += 0.1;
        }
        if is_key_pressed(KeyCode::Minus) {
            img_scale -= 0.1;
        }

        // palette panel size
        let panel = PanelSettings {
            padding: 3.0,
            inner_width: 208.0,
        };

        // Draw Image and panel
        panel.draw_image(&texture, img_scale);
        panel.draw_panel(&img);

        next_frame().await;
    }
}

struct PanelSettings {
    padding: f32,
    inner_width: f32,
    // panel_width: f32,
    // display: bool,
}

impl PanelSettings {
    #[inline]
    fn panel_width(&self) -> f32 {
        self.inner_width + self.padding
    }

    fn draw_image(&self, texture: &Texture2D, img_scale: f32) {
        let aspect_ratio = texture.width() / texture.height();
        let s =
            (screen_height() * aspect_ratio).min(screen_width() - self.panel_width()) * img_scale;
        draw_texture_ex(
            *texture,
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
        let topx = screen_width() - self.panel_width();
        draw_rectangle(topx, 0.0, self.panel_width(), screen_height(), BLACK);
        for (i, rgb) in img.palette().iter().enumerate() {
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
