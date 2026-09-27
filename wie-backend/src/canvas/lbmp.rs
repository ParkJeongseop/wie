use alloc::{boxed::Box, format, vec::Vec};

use bytemuck::{Pod, Zeroable, from_bytes, pod_collect_to_vec};

use wie_util::{Result, WieError};

use crate::canvas::{ArgbPixel, Color, Image, PixelType, Rgb332Pixel, Rgb565Pixel, VecImageBuffer};

// lcd bitmap file format for skvm
//
// header (little endian u32s) is followed by the pixel data and, when `mask` is set,
// a 1bpp transparency plane. 1bpp planes (grayscale pixels and the mask) pack eight
// rows per byte, column by column: byte `(y / 8) * width + x`, bit `y % 8` (LSB is
// the top row). A set mask bit makes the pixel transparent.

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LbmpHeader {
    descriptor: u32,
    r#type: u32,
    width: u32,
    height: u32,
    size: u32,
    mask: u32,
}

const HEADER_SIZE: usize = 24;

pub fn decode_lbmp(data: &[u8]) -> Result<Box<dyn Image>> {
    if data.len() < HEADER_SIZE {
        return Err(WieError::FatalError(format!("LBMP header truncated: {} bytes", data.len())));
    }
    let header: &LbmpHeader = from_bytes(&data[0..HEADER_SIZE]);
    let (width, height) = (header.width as usize, header.height as usize);
    let plane_len = height.div_ceil(8) * width;

    let pixel_len = match header.r#type {
        2 => header.size as usize * 2, // two 1bpp planes: high bit, low bit
        8 => width * height,
        16 => width * height * 2,
        x => return Err(WieError::Unimplemented(format!("Unsupported LBMP type {x}"))),
    };
    let mask_len = if header.mask != 0 { plane_len } else { 0 };
    if data.len() < HEADER_SIZE + pixel_len + mask_len {
        return Err(WieError::FatalError(format!(
            "LBMP data truncated: type {} {}x{} mask {} needs {} bytes, got {}",
            header.r#type,
            width,
            height,
            header.mask,
            HEADER_SIZE + pixel_len + mask_len,
            data.len()
        )));
    }
    let pixels = &data[HEADER_SIZE..HEADER_SIZE + pixel_len];
    let mask = (header.mask != 0).then(|| &data[HEADER_SIZE + pixel_len..HEADER_SIZE + pixel_len + mask_len]);

    if mask.is_none() {
        match header.r#type {
            8 => {
                return Ok(Box::new(VecImageBuffer::<Rgb332Pixel>::from_raw(
                    header.width,
                    header.height,
                    pixels.to_vec(),
                )));
            }
            16 => {
                return Ok(Box::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
                    header.width,
                    header.height,
                    pod_collect_to_vec(pixels),
                )));
            }
            _ => {}
        }
    }

    let color_at = |x: usize, y: usize| -> Color {
        match header.r#type {
            2 => {
                let stride = header.size as usize;
                let level = (plane_bit(&pixels[..stride], width, x, y) << 1) | plane_bit(&pixels[stride..], width, x, y);
                let gray = 255 - level * 85; // 4-level grayscale LCD: 0 = white, 3 = black
                Color {
                    a: 255,
                    r: gray,
                    g: gray,
                    b: gray,
                }
            }
            8 => Rgb332Pixel::to_color(pixels[y * width + x]),
            _ => Rgb565Pixel::to_color(u16::from_le_bytes([pixels[(y * width + x) * 2], pixels[(y * width + x) * 2 + 1]])),
        }
    };

    let mut raw = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let mut color = color_at(x, y);
            if let Some(mask) = mask
                && plane_bit(mask, width, x, y) != 0
            {
                color.a = 0;
            }
            raw.push(ArgbPixel::from_color(color));
        }
    }

    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(header.width, header.height, raw)))
}

fn plane_bit(plane: &[u8], width: usize, x: usize, y: usize) -> u8 {
    (plane[(y / 8) * width + x] >> (y % 8)) & 1
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn lbmp(r#type: u32, width: u32, height: u32, size: u32, mask: u32, body: &[u8]) -> Vec<u8> {
        let mut data = Vec::new();
        for word in [u32::from_le_bytes(*b"LBMP"), r#type, width, height, size, mask] {
            data.extend_from_slice(&word.to_le_bytes());
        }
        data.extend_from_slice(body);
        data
    }

    #[test]
    fn rgb332_mask_plane_is_column_packed_with_set_bits_transparent() {
        // 3x9: rows 0..8 in the first byte row, row 8 in the second
        let pixels = vec![0xff; 27];
        // column 0: rows 0 and 7 transparent; column 1: row 8 transparent; column 2 opaque
        let mask = vec![0b1000_0001, 0, 0, 0, 0b0000_0001, 0];
        let image = decode_lbmp(&lbmp(8, 3, 9, 27, 1, &[pixels, mask].concat())).unwrap();

        assert_eq!(image.get_pixel(0, 0).a, 0);
        assert_eq!(image.get_pixel(0, 7).a, 0);
        assert_eq!(image.get_pixel(0, 1).a, 255);
        assert_eq!(image.get_pixel(1, 8).a, 0);
        assert_eq!(image.get_pixel(1, 7).a, 255);
        assert_eq!(image.get_pixel(2, 8).a, 255);
        assert_eq!(
            (image.get_pixel(2, 8).r, image.get_pixel(2, 8).g, image.get_pixel(2, 8).b),
            (252, 252, 255)
        );
    }

    #[test]
    fn grayscale_planes_give_four_levels() {
        // 1x4, one byte per plane: high plane rows 2,3; low plane rows 1,3
        let image = decode_lbmp(&lbmp(2, 1, 4, 1, 0, &[0b1100, 0b1010])).unwrap();

        assert_eq!([0, 1, 2, 3].map(|y| image.get_pixel(0, y).r), [255, 170, 85, 0]);
        assert_eq!(image.get_pixel(0, 3).a, 255);
    }

    #[test]
    fn grayscale_mask_follows_both_planes() {
        let image = decode_lbmp(&lbmp(2, 2, 2, 2, 1, &[0b11, 0b11, 0b00, 0b00, 0b01, 0b10])).unwrap();

        assert_eq!(image.get_pixel(0, 0).a, 0);
        assert_eq!(image.get_pixel(0, 1).a, 255);
        assert_eq!(image.get_pixel(1, 0).a, 255);
        assert_eq!(image.get_pixel(1, 1).a, 0);
    }

    #[test]
    fn rgb565_with_mask_keeps_color() {
        let pixel = 0b1111_1000_0000_0000u16.to_le_bytes(); // pure red
        let image = decode_lbmp(&lbmp(16, 1, 1, 2, 1, &[pixel[0], pixel[1], 0b1])).unwrap();
        let color = image.get_pixel(0, 0);

        assert_eq!((color.a, color.r, color.g, color.b), (0, 255, 0, 0));
    }

    #[test]
    fn unmasked_images_decode_as_before() {
        let image = decode_lbmp(&lbmp(8, 2, 1, 2, 0, &[0b1110_0000, 0b0001_1100])).unwrap();

        assert_eq!(image.get_pixel(0, 0).r, 252);
        assert_eq!(image.get_pixel(1, 0).g, 252);
        assert!(decode_lbmp(&lbmp(8, 2, 1, 2, 1, &[0, 0])).is_err(), "missing mask plane is an error");
    }
}
