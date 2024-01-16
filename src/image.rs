use std::path::Path;

use imgref::ImgVec;
use macroquad::prelude::*;
use png::BitDepth;
use rgb::RGB8 as RGB;
/// Not to be confused with `lodepng::Image` or `macroquad::Image`
pub struct Image {
    image: ImgVec<u8>,
    texture: Option<Texture2D>,
    palette: Vec<RGB>,
    /// Transparency table, can be shorter than palette (but not longer)
    trns: Vec<u8>,
}

impl Image {
    pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
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
        let palette = png.info().palette.clone().map_or_else(
            || vec![RGB::default(); 2],
            |plte| {
                plte.chunks_exact(3)
                    .map(|c| RGB::new(c[0], c[1], c[2]))
                    .collect::<Vec<RGB>>()
            },
        );

        let trns = match png.info().trns.clone() {
            Some(c) => c.into_owned(),
            None => Vec::with_capacity(255),
        };

        let mut buf = vec![0; png.output_buffer_size()];
        let o_info = png.next_frame(&mut buf)?;
        let buf = match o_info.bit_depth {
            BitDepth::Sixteen => panic!("16-bit colormap png!?"),
            BitDepth::Eight => buf,
            bitdepth => unpack(&buf, bitdepth, o_info.width as usize),
        };

        Ok(Self::from_buffers(
            o_info.width,
            o_info.height,
            buf,
            palette,
            trns,
        ))
    }

    /// Creates image from raw u8 buffers.
    ///
    /// currently the palette has to be copied to a new vec,
    /// but this should be fixed later on.
    pub fn from_buffers(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        palette: Vec<RGB>,
        trns: Vec<u8>,
    ) -> Self {
        let image = ImgVec::new(pixels, width as usize, height as usize);

        Self {
            image,
            palette,
            texture: None,
            trns,
        }
    }

    pub fn palette(&self) -> &[RGB] {
        &self.palette
    }

    fn to_texture(&self) -> Texture2D {
        let image = self.image.as_ref();
        let width = image.width();
        let height = image.height();

        let mut buffer = Vec::with_capacity(width * height * 3);

        for i in image.pixels().map(|i| i as usize) {
            // Image shouldn't refer to colors not in the palette...
            for color in self.palette[i].iter() {
                buffer.push(color);
            }
            // ... but not all palette entries need to have transparency
            buffer.push(self.trns.get(i).copied().unwrap_or(255));
        }

        let t = Texture2D::from_rgba8(
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            &buffer,
        );
        t.set_filter(FilterMode::Nearest);
        t
    }

    // get a reference to the cache texture, creating it if not present.
    pub fn texture(&mut self) -> &Texture2D {
        if self.texture.is_none() {
            self.texture = Some(self.to_texture());
        }
        self.texture.as_ref().unwrap()
    }
}

/// Unpack from bitdepth under 8 bits to whole bytes.
///
/// Removes  padding at the end of scanlines.
/// See [RFC 2083 (section 2.3)](https://datatracker.ietf.org/doc/html/rfc2083#page-7)
pub fn unpack(packed: &[u8], bitdepth: BitDepth, line_width: usize) -> Vec<u8> {
    match bitdepth {
        BitDepth::Sixteen => panic!("cannot unpack 16 bits"),
        BitDepth::Eight => {
            warn!("unpacking from 8 bits to 8 bits (unnecessary allocation");
            packed.to_owned()
        }
        bitdepth => {
            let bitdepth = bitdepth as u8;
            let mut buf_w = Vec::<u8>::with_capacity(packed.len() / (8 / bitdepth) as usize);
            // modulus for extracting lower `bitdepth` bits.
            let modulus = 2u8.pow(u32::from(bitdepth));
            // because the modulus is a power of 2, the preceding number
            // consists of `bitdepth` repeated ones
            // (i.e.: bitdepth = 4 => mask = 0b0000_1111)
            // try it with a numeral system/programming mode calculator!
            let mask = modulus - 1;
            let px_per_byte = 8 / bitdepth as usize;
            let mut line_pos = 0;
            for byte in packed {
                // iterate over pixel indices in byte
                // ends early if the pixels are "wasted bytes"
                for pxi in 0..px_per_byte.min(line_width - line_pos) {
                    // biggest offset (leftmost bits) goes first. last offset is 0.
                    let offset = (px_per_byte - pxi - 1) * bitdepth as usize;
                    let px = (byte >> offset) & mask;
                    buf_w.push(px);
                    line_pos += 1;
                }
                if line_pos >= line_width {
                    line_pos = 0;
                }
            }
            buf_w
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use png::BitDepth;

    #[test]
    fn unpack_test_2() {
        let v = vec![0b1111_1111; 8];
        let x = unpack(&v, BitDepth::Two, 8 * 4);
        assert_eq!(x, [0b0000_0011; 8 * 4]);
    }

    #[test]
    fn unpack_test_4() {
        let v = vec![0b1111_1111; 8];
        let x = unpack(&v, BitDepth::Four, 8 * 2);
        assert_eq!(x, [0b0000_1111; 8 * 2]);
    }

    #[test]
    fn unpack_test_4_2() {
        let v = vec![0b1011_1011; 8];
        let x = unpack(&v, BitDepth::Four, 8 * 2);
        assert_eq!(x, [0b0000_1011; 8 * 2]);
    }

    #[test]
    fn unpack_test_4_width() {
        let v = [0x01, 0x2a, 0x34, 0x5b, 0x67, 0x8c];
        let x = unpack(&v, BitDepth::Four, 3);
        let res = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(x, res);
    }
}
