use alloc::{boxed::Box, vec, vec::Vec};

use bytemuck::{Pod, Zeroable};
use wie_backend::canvas::{ArgbPixel, Color, Image, PixelType, Rgb565Pixel, VecImageBuffer, decode_image};
use wie_util::{Result, WieError, read_generic, write_generic};

use wipi_types::wipic::{WIPICFramebuffer, WIPICIndirectPtr, WIPICWord};

use crate::{api::graphics::framebuffer::FrameBuffer, context::WIPICContext};

/// `MC_GrpImage` as the specification and the handset's own macros lay it out: the image's pixels
/// and mask are framebuffer *handles* (`MC_GrpFrameBuffer`), not framebuffers inlined. A title that
/// reads an image through those macros follows word zero to a framebuffer record and switches on
/// its depth field (`(bpp << 24) >> 27`, 2 for 16-bit and 4 for 32-bit surfaces); with the
/// framebuffer inlined, word zero was the width and 폴라폴리2007 dereferenced 74.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ImageRecord {
    pub img: WIPICIndirectPtr,
    pub mask: WIPICIndirectPtr,
    pub loop_count: WIPICWord,
    pub delay: WIPICWord,
    pub animated: WIPICWord,
    pub buf: WIPICIndirectPtr,
    pub offset: WIPICWord,
    pub current: WIPICWord,
    pub len: WIPICWord,
}

/// An encoded image as the handset keeps one: RGB565 pixels (little endian) and one alpha byte per
/// pixel, 255 opaque. A transparent pixel keeps its own colour, since the decode is not
/// premultiplied: titles that blit an image themselves key on that colour.
pub struct Rgb565Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub alpha: Vec<u8>,
}

impl Rgb565Image {
    pub fn decode(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<Self> {
        let address = context.data_ptr(buf)?.checked_add(offset).ok_or(WieError::AllocationFailure)?;
        let mut data = vec![0; len as usize];
        context.read_bytes(address, &mut data)?;
        let image = decode_image(&data)?;

        let (width, height) = (image.width(), image.height());
        let count = (width as usize).checked_mul(height as usize).ok_or(WieError::AllocationFailure)?;
        let mut pixels = Vec::with_capacity(count * 2);
        let mut alpha = Vec::with_capacity(count);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let color = image.get_pixel(x, y);
                pixels.extend_from_slice(&Rgb565Pixel::from_color(color).to_le_bytes());
                alpha.push(color.a);
            }
        }

        Ok(Self {
            width,
            height,
            pixels,
            alpha,
        })
    }

    pub fn is_opaque(&self) -> bool {
        self.alpha.iter().all(|&a| a == 0xff)
    }

    /// Composes RGB565 pixels and their alpha bytes back into an image the platform draws with.
    pub fn compose(width: u32, height: u32, pixels: &[u8], alpha: &[u8]) -> Box<dyn Image> {
        let argb = pixels
            .chunks_exact(2)
            .zip(alpha)
            .map(|(rgb, &a)| {
                let color = Rgb565Pixel::to_color(u16::from_le_bytes([rgb[0], rgb[1]]));
                ArgbPixel::from_color(Color { a, ..color })
            })
            .collect();

        Box::new(VecImageBuffer::<ArgbPixel>::from_raw(width, height, argb))
    }
}

fn alloc_framebuffer(context: &mut dyn WIPICContext, width: u32, height: u32, bpp: u32, data: &[u8]) -> Result<WIPICIndirectPtr> {
    let buf = context.alloc(u32::try_from(data.len()).map_err(|_| WieError::AllocationFailure)?)?;
    context.write_bytes(context.data_ptr(buf)?, data)?;
    let record = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(
        context,
        context.data_ptr(record)?,
        WIPICFramebuffer {
            width,
            height,
            bpl: width * bpp / 8,
            bpp,
            buf,
        },
    )?;

    Ok(record)
}

/// Builds an `MC_GrpImage`: RGB565 pixels in `img`, the handset's own depth (a title that blits an
/// image itself writes it into the 16-bit screen with the depth it reads off `img`), and, when
/// the encoding has any transparency, its alpha in `mask`, one byte per pixel with 255 opaque —
/// what 폴라폴리2007's blitter blends with (`src + (dst - src) * (255 - mask) / 255`).
pub fn create_wipi_image(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<ImageRecord> {
    let image = Rgb565Image::decode(context, buf, offset, len)?;
    let img = alloc_framebuffer(context, image.width, image.height, 16, &image.pixels)?;
    let mask = if image.is_opaque() {
        WIPICIndirectPtr(0)
    } else {
        alloc_framebuffer(context, image.width, image.height, 8, &image.alpha)?
    };

    Ok(ImageRecord {
        img,
        mask,
        loop_count: 0,
        delay: 0,
        animated: 0,
        buf,
        offset,
        current: 0,
        len,
    })
}

pub fn decode_image_framebuffer(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<FrameBuffer> {
    let address = context.data_ptr(buf)?.checked_add(offset).ok_or(WieError::AllocationFailure)?;
    let mut data = vec![0; len as usize];
    context.read_bytes(address, &mut data)?;
    tracing::debug!(
        "decode_image_framebuffer(buf {:#x} -> {address:#x}, offset {offset:#x}, len {len:#x}): {:02x?}",
        buf.0,
        &data[..data.len().min(16)]
    );
    let image = decode_image(&data)?;
    FrameBuffer::from_image(context, &*image)
}

/// The image an `MC_GrpImage` holds, with its mask's alpha when it has one, for the platform's own
/// draws.
pub fn image_with_mask(context: &mut dyn WIPICContext, record: &ImageRecord) -> Result<Box<dyn Image>> {
    let img = FrameBuffer(read_generic(context, context.data_ptr(record.img)?)?);
    if record.mask.0 == 0 || img.0.bpp != 16 {
        return img.image(context);
    }
    let mask: WIPICFramebuffer = read_generic(context, context.data_ptr(record.mask)?)?;

    let count = (img.0.width as usize) * (img.0.height as usize);
    let mut pixels = vec![0u8; count * 2];
    context.read_bytes(context.data_ptr(img.0.buf)?, &mut pixels)?;
    let mut alpha = vec![0u8; count];
    context.read_bytes(context.data_ptr(mask.buf)?, &mut alpha)?;

    Ok(Rgb565Image::compose(img.0.width, img.0.height, &pixels, &alpha))
}
