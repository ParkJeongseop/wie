#![allow(clippy::too_many_arguments)]

use alloc::{string::String, vec};

use wie_backend::canvas::{Canvas, Clip, Color, Image, PixelType, Rgb8Pixel};
use wie_util::{Result, read_null_terminated_string_bytes};

use crate::{WIPICContext, api::graphics::FrameBuffer};

pub fn read_text(context: &dyn WIPICContext, address: u32, length: i32) -> Result<Option<String>> {
    let bytes = if length == -1 {
        read_null_terminated_string_bytes(context, address)?
    } else if length >= 0 {
        let mut bytes = vec![0; length as usize];
        context.read_bytes(address, &mut bytes)?;
        bytes
    } else {
        return Ok(None);
    };

    Ok(Some(encoding_rs::EUC_KR.decode(&bytes).0.into_owned()))
}

/// Whether the `length` bytes at `address` end on the first byte of a two-byte character.
pub fn ends_inside_character(context: &dyn WIPICContext, address: u32, length: i32) -> Result<bool> {
    let mut bytes = vec![0; length as usize];
    context.read_bytes(address, &mut bytes)?;

    let mut end = 0;
    while end < bytes.len() {
        end += if (0x81..=0xfe).contains(&bytes[end]) { 2 } else { 1 };
    }
    Ok(end > bytes.len())
}

/// Runs a draw on a canvas of the whole framebuffer at `alpha`: 255 replaces what the draw covers and anything lower
/// blends it over what is there.
fn write_canvas<F>(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, alpha: u8, operation: F) -> Result<()>
where
    F: FnOnce(&mut dyn Canvas),
{
    let mut canvas = framebuffer.canvas(context, 0..framebuffer.0.height)?;
    canvas.set_alpha(alpha);
    operation(&mut **canvas);
    canvas.flush()
}

/// Runs a draw that stays within the `height` rows from `y` on a canvas of those rows alone, handing it `y` and `clip` as
/// they are on that canvas. A title that builds its screen out of small images otherwise spends its time copying the
/// whole framebuffer out of the guest and back for each of them (럭키당구).
fn write_rows<F>(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, y: i32, height: u32, clip: Clip, alpha: u8, operation: F) -> Result<()>
where
    F: FnOnce(&mut dyn Canvas, i32, Clip),
{
    let top = (y as i64).max(0);
    let bottom = (y as i64 + height as i64).min(framebuffer.0.height as i64);
    let rows = if top < bottom { top as u32..bottom as u32 } else { 0..0 };

    let mut clip = clip.intersect(&Clip {
        y: rows.start as i32,
        height: rows.len() as u32,
        ..clip
    });
    clip.y -= rows.start as i32;

    let mut canvas = framebuffer.canvas(context, rows.clone())?;
    canvas.set_alpha(alpha);
    operation(&mut **canvas, y - rows.start as i32, clip);
    canvas.flush()
}

pub fn put_pixel(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, x: i32, y: i32, color: Color, clip: Clip, alpha: u8) -> Result<()> {
    write_rows(context, framebuffer, y, 1, clip, alpha, |canvas, y, clip| {
        canvas.put_pixel(x, y, color, clip)
    })
}

pub fn fill_rect(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Color,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    write_rows(context, framebuffer, y, height, clip, alpha, |canvas, y, clip| {
        canvas.fill_rect(x, y, width, height, color, clip)
    })
}

pub fn draw_line(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: Color,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    write_canvas(context, framebuffer, alpha, |canvas| canvas.draw_line(x1, y1, x2, y2, color, clip))
}

pub fn draw_rect(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Color,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    write_canvas(context, framebuffer, alpha, |canvas| canvas.draw_rect(x, y, width, height, color, clip))
}

pub fn draw_arc(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    start_angle: i32,
    arc_angle: i32,
    color: Color,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    write_canvas(context, framebuffer, alpha, |canvas| {
        canvas.draw_arc(x, y, width, height, start_angle, arc_angle, color, clip)
    })
}

pub fn fill_arc(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    start_angle: i32,
    arc_angle: i32,
    color: Color,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    write_canvas(context, framebuffer, alpha, |canvas| {
        canvas.fill_arc(x, y, width, height, start_angle, arc_angle, color, clip)
    })
}

pub fn draw_image(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    image: &dyn Image,
    source_x: i32,
    source_y: i32,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    write_rows(context, framebuffer, y, height, clip, alpha, |canvas, y, clip| {
        canvas.draw(x, y, width, height, image, source_x, source_y, clip)
    })
}

pub fn copy_area(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source_x: i32,
    source_y: i32,
    clip: Clip,
) -> Result<()> {
    let image = framebuffer.image(context)?;
    write_rows(context, framebuffer, y, height, clip, 255, |canvas, y, clip| {
        canvas.draw(x, y, width, height, &*image, source_x, source_y, clip)
    })
}

pub fn copy_framebuffer(
    context: &mut dyn WIPICContext,
    destination: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source: &FrameBuffer,
    source_x: i32,
    source_y: i32,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    let image = source.image(context)?;
    write_rows(context, destination, y, height, clip, alpha, |canvas, y, clip| {
        canvas.draw(x, y, width, height, &*image, source_x, source_y, clip)
    })
}

/// Draws `string` with its 12px cell top at `y` (ink starts at `y`, like the original
/// full-cell bitmap fonts). Callers convert from their platform's convention first.
pub fn draw_text(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    string: &str,
    x: i32,
    y: i32,
    color: Color,
    clip: Clip,
    alpha: u8,
) -> Result<()> {
    let font = context.system().platform().font().clone();
    write_canvas(context, framebuffer, alpha, |canvas| {
        canvas.draw_text(
            &font,
            string,
            x,
            y,
            super::FONT_SIZE,
            wie_backend::canvas::TextAlignment::Left,
            color,
            clip,
        )
    })
}

fn rgb_row_bytes(width: i32, stride: i32) -> Option<usize> {
    if width <= 0 || stride <= 0 {
        return None;
    }

    let row_bytes = (width as usize).checked_mul(4)?;
    (stride as usize >= row_bytes).then_some(row_bytes)
}

pub fn get_rgb_pixels(
    context: &mut dyn WIPICContext,
    image: &dyn Image,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    destination: u32,
    destination_bpl: i32,
) -> Result<()> {
    if height <= 0 {
        return Ok(());
    }
    let Some(row_bytes) = rgb_row_bytes(width, destination_bpl) else {
        return Ok(());
    };

    let mut row = vec![0; row_bytes];
    for row_index in 0..height {
        for column in 0..width {
            let source_x = x.wrapping_add(column);
            let source_y = y.wrapping_add(row_index);
            let color = if source_x < 0 || source_y < 0 || source_x >= image.width() as i32 || source_y >= image.height() as i32 {
                Color { a: 0, r: 0, g: 0, b: 0 }
            } else {
                image.get_pixel(source_x, source_y)
            };
            let offset = column as usize * 4;
            row[offset..offset + 4].copy_from_slice(&Rgb8Pixel::from_color(color).to_le_bytes());
        }
        let address = destination
            .checked_add(
                (row_index as u32)
                    .checked_mul(destination_bpl as u32)
                    .ok_or(wie_util::WieError::AllocationFailure)?,
            )
            .ok_or(wie_util::WieError::AllocationFailure)?;
        context.write_bytes(address, &row)?;
    }
    Ok(())
}

pub fn set_rgb_pixels(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    source: u32,
    source_bpl: i32,
    clip: Clip,
) -> Result<()> {
    if height <= 0 {
        return Ok(());
    }
    let Some(row_bytes) = rgb_row_bytes(width, source_bpl) else {
        return Ok(());
    };
    let Some(total_bytes) = row_bytes.checked_mul(height as usize) else {
        return Ok(());
    };

    let mut pixels = vec![0; total_bytes];
    for row_index in 0..height {
        let address = source
            .checked_add(
                (row_index as u32)
                    .checked_mul(source_bpl as u32)
                    .ok_or(wie_util::WieError::AllocationFailure)?,
            )
            .ok_or(wie_util::WieError::AllocationFailure)?;
        let offset = row_index as usize * row_bytes;
        context.read_bytes(address, &mut pixels[offset..offset + row_bytes])?;
    }

    write_rows(context, framebuffer, y, height as u32, clip, 255, |canvas, y, clip| {
        for row_index in 0..height {
            let row_offset = row_index as usize * row_bytes;
            let row = &pixels[row_offset..row_offset + row_bytes];
            for column in 0..width {
                let offset = column as usize * 4;
                let rgb = u32::from_le_bytes([row[offset], row[offset + 1], row[offset + 2], row[offset + 3]]);
                canvas.put_pixel(x.wrapping_add(column), y.wrapping_add(row_index), Rgb8Pixel::to_color(rgb), clip);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use wie_backend::canvas::{Clip, Color};
    use wie_util::{ByteRead, ByteWrite, Result};

    use crate::{
        api::graphics::{
            FrameBuffer,
            primitives::{fill_rect, get_rgb_pixels, put_pixel, set_rgb_pixels},
        },
        context::{WIPICContext, test::TestContext},
    };

    #[test]
    fn drawing_primitives_write_the_guest_framebuffer() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 4, 4, 16)?;
        let clip = Clip {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let red = Color {
            a: 0xff,
            r: 0xff,
            g: 0,
            b: 0,
        };

        fill_rect(&mut context, &framebuffer, 1, 1, 2, 2, red, clip, 255)?;
        put_pixel(&mut context, &framebuffer, 0, 0, red, clip, 255)?;

        let image = framebuffer.image(&mut context)?;
        assert_eq!(image.get_pixel(0, 0).r, 255);
        assert_eq!(image.get_pixel(1, 1).r, 255);
        assert_eq!(image.get_pixel(3, 3).r, 0);
        Ok(())
    }

    #[test]
    fn rgb_pixels_use_the_wipi_little_endian_layout() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 1, 1, 16)?;
        put_pixel(
            &mut context,
            &framebuffer,
            0,
            0,
            Color {
                a: 0xff,
                r: 0xff,
                g: 0,
                b: 0,
            },
            Clip {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            255,
        )?;
        let image = framebuffer.image(&mut context)?;

        get_rgb_pixels(&mut context, &*image, 0, 0, 1, 1, 0x1000, 4)?;
        let mut bytes = [0; 4];
        context.read_bytes(0x1000, &mut bytes)?;
        assert_eq!(u32::from_le_bytes(bytes), 0x00ff_0000);
        Ok(())
    }

    #[test]
    fn rgb_row_bytes_rejects_overflow_and_short_strides() {
        assert_eq!(super::rgb_row_bytes(i32::MAX, i32::MAX), None);
        assert_eq!(super::rgb_row_bytes(4, 15), None);
        assert_eq!(super::rgb_row_bytes(4, 16), Some(16));
    }

    #[test]
    fn set_rgb_pixels_wraps_guest_coordinates() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 1, 1, 16)?;
        let source = context.alloc(8)?;
        context.write_bytes(context.data_ptr(source)?, &[0, 0, 0, 0, 0, 0, 0, 0])?;

        set_rgb_pixels(
            &mut context,
            &framebuffer,
            i32::MAX,
            0,
            2,
            1,
            source.0,
            8,
            Clip {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        )
    }
}
