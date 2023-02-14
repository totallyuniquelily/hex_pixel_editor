use std::io::Write;

use imgref::{ImgRef, ImgVec};
use macroquad::prelude::*;
use png::BitDepth;
use rgb::{ComponentBytes, RGB8 as RGB};
/// Not to be confused with `lodepng::Image` or `macroquad::Image`
pub struct Image {
    palette: Vec<RGB>,
    image: ImgVec<u8>,
    /// Transparency table, can be shorter than palette (but not longer)
    trns: Vec<u8>,
}

impl Image {
    // Create a blank image
    pub fn new(x: usize, y: usize) -> Self {
        // I know this is premature optimisation but I couldn't stop myself.
        let mut palette = Vec::with_capacity(255);
        let mut trns = Vec::with_capacity(255);
        palette.extend([RGB::default(); 2]);
        trns.push(0);
        Self {
            palette,
            image: ImgVec::new(vec![0; x * y], x, y),
            trns,
        }
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
            trns,
        }
    }

    pub fn palette(&self) -> &[RGB] {
        &self.palette
    }
    pub fn image(&self) -> ImgRef<u8> {
        self.image.as_ref()
    }
    pub fn trns(&self) -> &[u8] {
        &self.trns
    }

    pub fn set_pixel(&mut self, index: (usize, usize), color: u8) {
        assert!((color as usize) < self.palette.len());
        self.image.as_mut()[index] = color;
    }

    pub fn set_color(&mut self, index: u8, color: RGB) {
        self.palette[index as usize] = color;
    }

    pub fn push_color(&mut self, color: RGB) {
        self.palette.push(color);
    }

    pub fn set_transparency(&mut self, index: u8, transparency: u8) {
        let index = index as usize;
        assert!(index < self.palette.len());
        if self.trns.len() >= index {
            self.trns.resize(index + 1, 255);
        }
        self.trns[index] = transparency;
        self.shrink_trns();
    }

    /// Remove trailing opaque entries from trns. Doesn't actually shrink the `Vec` capacity.
    pub fn shrink_trns(&mut self) {
        let trns = &mut self.trns;
        while let Some(n) = trns.pop() {
            if n < 255 {
                trns.push(n);
                break;
            }
        }
    }

    pub fn to_texture(&self) -> Texture2D {
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

    pub fn encode<W: Write>(&self, w: W) {
        use png::*;
        let img = self.image.as_ref();
        let mut encoder = Encoder::new(w, img.width() as u32, img.height() as u32);
        encoder.set_color(ColorType::Indexed);
        encoder.set_palette(self.palette.as_bytes());
        encoder.set_trns(&self.trns);

        // Packing is not supported yet.
        encoder.set_depth(BitDepth::Eight);

        // No filtering is recommended for indexed images
        // (RFC 2083 section 9.6)
        // https://datatracker.ietf.org/doc/html/rfc2083#page-49
        encoder.set_filter(FilterType::NoFilter);

        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&self.image.as_ref().to_contiguous_buf().0)
            .unwrap();
    }
}

impl Default for Image {
    fn default() -> Self {
        /// The width/height of the image
        const N: usize = 16;
        Self::new(N, N)
    }
}

/// Unpack from bitdepth under 8 bits to whole bytes
pub fn unpack(packed: &[u8], bitdepth: BitDepth) -> Vec<u8> {
    match bitdepth {
        BitDepth::Sixteen => panic!("cannot unpack 16 bits"),
        BitDepth::Eight => {
            warn!("unpacking from 8 bits to 8 bits (unnecessary allocation");
            packed.to_owned()
        }
        bitdepth => {
            let bitdepth = bitdepth as u8;
            let mut buf_w = Vec::<u8>::with_capacity(packed.len() / (8 / bitdepth) as usize);
            let modulus = 2u8.pow(bitdepth as u32);
            for byte in packed {
                for sub_byte in 0..8 / bitdepth {
                    let px = (byte >> (sub_byte * bitdepth)) % modulus;
                    buf_w.push(px);
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
    fn unpack_test() {
        let v = vec![0b11111111; 8];
        let x = unpack(&v, BitDepth::Two);
        assert_eq!(x, [0b0000_0011; 8 * 4]);
    }
}
