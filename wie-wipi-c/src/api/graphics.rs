mod framebuffer;
mod grp_context;
mod image;
pub mod primitives;

pub use framebuffer::FrameBuffer;
pub use grp_context::GraphicsContextLayout;
pub use image::{Rgb565Image, decode_image_framebuffer};

use alloc::{vec, vec::Vec};
use core::mem::size_of;

use wie_backend::{
    Event,
    canvas::{Clip, Color, PixelType, Rgb565Pixel, encode_bmp_region, string_width},
};
use wie_util::{Result, read_generic, write_generic};

use wipi_types::wipic::{WIPICDisplayInfo, WIPICFramebuffer, WIPICGraphicsContext, WIPICIndirectPtr, WIPICWord};

use crate::{
    context::WIPICContext,
    globals::{SCREEN_FRAMEBUFFER_DIRTY_PTR, SCREEN_FRAMEBUFFER_PTR},
};

use self::{
    grp_context::{WIPICGraphicsContextIdx, read_graphics_context, write_graphics_context},
    image::{ImageRecord, create_wipi_image, image_with_mask},
};

const FRAMEBUFFER_DEPTH: u32 = 16; // XXX hardcode to 16bpp as some game requires 16bpp framebuffer

/// What `MC_grpSetContext` leaves as the context's pixel operation when it is given an alpha: the platform's own
/// blend, which no title's function can be mistaken for.
const ALPHA_PIXEL_OP: WIPICWord = 1;

/// Intersects a draw's own rectangle with the graphics context's clip. The record keeps the clip as
/// [left, top, right, bottom], the right and bottom edges outside it, in the destination's
/// coordinates (as wide as the carrier's layout makes them); an empty one, which is what `MC_grpInitContext` leaves and what
/// `MC_grpSetContext` with a null array sets, means "no clip". The KTF draws ignored it, so a title
/// that clips one cell out of a sprite sheet painted the whole sheet: 짜요짜요타이쿤3's logo smeared
/// across its splash and its menus filled with the neighbouring cells.
fn with_context_clip(context: &dyn WIPICContext, pgc: WIPICWord, clip: Clip) -> Result<Clip> {
    if pgc == 0 {
        return Ok(clip);
    }
    let gctx = read_graphics_context(context, pgc)?;
    let [left, top, right, bottom] = gctx.clip.map(|value| i32::from(value as i16));
    if right <= left || bottom <= top {
        return Ok(clip);
    }

    Ok(clip.intersect(&Clip {
        x: left,
        y: top,
        width: (right - left) as u32,
        height: (bottom - top) as u32,
    }))
}

/// Moves a draw's point to where the graphics context's origin puts it. `MC_GRP_CONTEXT_OFFSET_IDX` sets
/// "the origin of the relative coordinates", so what a title passes to a draw is measured from there,
/// while the clip stays in the destination's own coordinates. The KTF draws ignored it: 액션히어로3D
/// paints its status panel at (0, 0) with the origin at (0, 237), and the panel landed on the top of the
/// screen while the bottom kept the loading screen's last frame.
fn from_context_origin(context: &dyn WIPICContext, pgc: WIPICWord, x: i32, y: i32) -> Result<(i32, i32)> {
    if pgc == 0 {
        return Ok((x, y));
    }
    let gctx = read_graphics_context(context, pgc)?;
    let [origin_x, origin_y] = gctx.offset.map(|value| i32::from(value as i16));

    Ok((x.wrapping_add(origin_x), y.wrapping_add(origin_y)))
}

/// What a title's pixel operation has yet to decide: the rows a draw reaches as they were, and each pixel the draw
/// writes, by where it is in them, with the value the draw has for it and how much of it the draw covers, out of 255.
struct OperatedPixels {
    function: WIPICWord,
    param1: WIPICWord,
    top: u32,
    rows: Vec<u8>,
    written: Vec<(usize, WIPICWord, u8)>,
}

fn pixel_value(bytes: &[u8]) -> WIPICWord {
    bytes.iter().rev().fold(0, |value, byte| (value << 8) | WIPICWord::from(*byte))
}

/// Runs a draw the way the graphics context's pixel operation has it written; `reach` is as far as the draw can get.
/// "The value this function returns is what every drawing function finally writes."
///
/// Giving a context an alpha "also sets its pixel operation and that operation's parameter", so the alpha (255
/// replaces what is there, 0 leaves it) is in force for as long as that operation is the context's. A record that
/// was only zeroed, or one whose title put the operation back to none after an alpha, draws as it is. The KTF draws
/// ignored the alpha altogether: 겟앰프드's intro swept opaque white bars over the screen instead of a sheen,
/// 폴라폴리2007's rule sheet hid the board it lies on, and the fades of 메이플스토리 and 크로노스윙 went black in
/// one step.
///
/// A title's own operation is asked about every pixel the draw writes, with that pixel, the one it lands on and the
/// context's parameter. 랩업만이살길 and 럭키당구 colour their bitmap fonts this way: the operation keeps what is
/// there under the glyph's key colour and answers the text colour for the rest, and without it each glyph was a block
/// of the key colour. 액션히어로3D blends its menu panel with one and 헬싱 passes every draw through one. The draw is
/// painted over black and over white to find the pixels it writes, and the answers come from
/// [`write_operated_pixels`]; a pixel that the edge of a glyph covers in part gets that share of its answer.
fn draw_through_context<F>(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    pgc: WIPICWord,
    reach: Clip,
    draw: F,
) -> Result<Option<OperatedPixels>>
where
    F: Fn(&mut dyn WIPICContext, u8) -> Result<()>,
{
    if pgc == 0 {
        draw(context, 255)?;

        return Ok(None);
    }
    let gctx = read_graphics_context(context, pgc)?;
    match gctx.pixel_op_func_ptr {
        0 => draw(context, 255)?,
        ALPHA_PIXEL_OP => draw(context, gctx.alpha.min(255) as u8)?,
        function => {
            let reach = reach.intersect(&Clip {
                x: 0,
                y: 0,
                width: framebuffer.0.width,
                height: framebuffer.0.height,
            });
            if reach.width == 0 || reach.height == 0 {
                return Ok(None);
            }
            let lines = reach.y as u32..reach.y as u32 + reach.height;
            let rows: Vec<u8> = framebuffer.data(context, lines.clone())?;
            let mut painted = Vec::new();
            for background in [0x00, 0xff] {
                framebuffer.write_rows(context, lines.start, &vec![background; rows.len()])?;
                draw(context, 255)?;
                painted.push(framebuffer.data::<u8>(context, lines.clone())?);
            }
            framebuffer.write_rows(context, lines.start, &rows)?;

            let size = (framebuffer.0.bpp / 8) as usize;
            let stride = framebuffer.0.width as usize * size;
            let written = (0..reach.height as usize)
                .flat_map(|row| {
                    let start = row * stride + reach.x as usize * size;

                    (start..start + reach.width as usize * size).step_by(size)
                })
                .filter_map(|offset| {
                    let over_black = pixel_value(&painted[0][offset..offset + size]);
                    let over_white = pixel_value(&painted[1][offset..offset + size]);
                    if over_black == over_white {
                        return Some((offset, over_black, 255));
                    }
                    // What shows of the white is the share of the pixel the draw left alone; the rest is the draw's,
                    // and over black it came out as that share of its colour.
                    let (black, white) = (framebuffer.pixel_to_color(over_black), framebuffer.pixel_to_color(over_white));
                    let coverage = 255 - white.g.saturating_sub(black.g);
                    let whole = |channel: u8| (u32::from(channel) * 255 / u32::from(coverage)).min(255) as u8;

                    (coverage != 0).then(|| {
                        let source = Color {
                            a: 255,
                            r: whole(black.r),
                            g: whole(black.g),
                            b: whole(black.b),
                        };

                        (offset, framebuffer.color_to_pixel(source), coverage)
                    })
                })
                .collect();

            return Ok(Some(OperatedPixels {
                function,
                param1: gctx.param1,
                top: lines.start,
                rows,
                written,
            }));
        }
    }

    Ok(None)
}

async fn write_operated_pixels(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, pixels: Option<OperatedPixels>) -> Result<()> {
    let Some(mut pixels) = pixels else {
        return Ok(());
    };

    let size = (framebuffer.0.bpp / 8) as usize;
    for (offset, source, coverage) in pixels.written {
        let bytes = &mut pixels.rows[offset..offset + size];
        let existing = pixel_value(bytes);
        let answer = context.call_function(pixels.function, &[source, existing, pixels.param1]).await?;
        let pixel = if coverage == 255 {
            answer
        } else {
            let (under, over) = (framebuffer.pixel_to_color(existing), framebuffer.pixel_to_color(answer));
            let mix = |under: u8, over: u8| ((u32::from(under) * u32::from(255 - coverage) + u32::from(over) * u32::from(coverage)) / 255) as u8;

            framebuffer.color_to_pixel(Color {
                a: 255,
                r: mix(under.r, over.r),
                g: mix(under.g, over.g),
                b: mix(under.b, over.b),
            })
        };
        bytes.copy_from_slice(&pixel.to_le_bytes()[..size]);
    }

    framebuffer.write_rows(context, pixels.top, &pixels.rows)
}

pub async fn fill_polygon(
    _context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x_points: WIPICWord,
    y_points: WIPICWord,
    n_points: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::warn!("stub MC_grpFillPolygon({:#x}, {x_points:#x}, {y_points:#x}, {n_points}, {pgc:#x})", dst.0);

    Ok(())
}

pub async fn get_screen_framebuffer(context: &mut dyn WIPICContext, a0: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetScreenFrameBuffer({a0:#x})");

    let framebuffer_ptr: u32 = read_generic(context, SCREEN_FRAMEBUFFER_PTR)?;
    if framebuffer_ptr != 0 {
        return Ok(WIPICIndirectPtr(framebuffer_ptr));
    }

    let (width, height) = {
        let platform = context.system().platform();
        let screen = platform.screen();
        (screen.width(), screen.height())
    };

    let framebuffer = FrameBuffer::new(context, width, height, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer.0)?;
    write_generic(context, SCREEN_FRAMEBUFFER_PTR, memory.0)?;

    Ok(memory)
}

pub async fn init_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpInitContext({p_grp_ctx:#x})");

    let grp_ctx = WIPICGraphicsContext::default();
    write_graphics_context(context, p_grp_ctx, grp_ctx)?;
    Ok(())
}

pub async fn set_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpSetContext({p_grp_ctx:#x}, {op:?}, {pv:#x})");

    let mut grp_ctx = read_graphics_context(context, p_grp_ctx)?;
    match op {
        // A null clip clears it: a context that was never given one reads as zeroes, which means
        // "no clip" (MC_grpInitContext zeroes the record). 짜요짜요타이쿤3 resets its clip this way
        // before every frame and the null was read as an array.
        WIPICGraphicsContextIdx::ClipIdx => {
            // MC_GRP_CONTEXT_CLIP_MASK is cleared when clipping is disabled.
            if pv == 0 {
                grp_ctx.clip = [0; 4];
                grp_ctx.mask &= !1;
            } else {
                let clip: [i32; 4] = read_generic(context, pv)?;
                grp_ctx.clip = clip.map(|value| value as u16);
                grp_ctx.mask |= 1;
            }
        }
        WIPICGraphicsContextIdx::FgPixelIdx => {
            grp_ctx.fgpxl = pv as _;
        }
        WIPICGraphicsContextIdx::BgPixelIdx => {
            grp_ctx.bgpxl = pv as _;
        }
        WIPICGraphicsContextIdx::TransPixelIdx => {
            grp_ctx.transpxl = pv as _;
        }
        WIPICGraphicsContextIdx::AlphaIdx => {
            grp_ctx.alpha = pv as _;
            grp_ctx.pixel_op_func_ptr = ALPHA_PIXEL_OP;
            grp_ctx.param1 = pv;
        }
        WIPICGraphicsContextIdx::PixelopIdx => {
            grp_ctx.pixel_op_func_ptr = pv;
        }
        WIPICGraphicsContextIdx::PixelParam1Idx => {
            grp_ctx.param1 = pv;
        }
        WIPICGraphicsContextIdx::FontIdx => {
            grp_ctx.font = pv;
        }
        WIPICGraphicsContextIdx::StyleIdx => {
            grp_ctx.style = pv;
        }
        WIPICGraphicsContextIdx::OffsetIdx if pv == 0 => {
            grp_ctx.offset = [0; 2];
        }
        WIPICGraphicsContextIdx::OffsetIdx => {
            let offset: [i32; 2] = read_generic(context, pv)?;
            grp_ctx.offset = offset.map(|value| value as u16);
        }
        _ => {
            tracing::warn!("MC_grpSetContext({p_grp_ctx:#x}, {op:?}, {pv:#x}): ignoring invalid op");
        }
    }
    write_graphics_context(context, p_grp_ctx, grp_ctx)?;

    Ok(())
}

pub async fn get_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpGetContext({p_grp_ctx:#x}, {op:?}, {pv:#x})");

    let grp_ctx = read_graphics_context(context, p_grp_ctx)?;
    match op {
        WIPICGraphicsContextIdx::ClipIdx => write_generic(context, pv, grp_ctx.clip.map(|value| i32::from(value as i16)))?,
        WIPICGraphicsContextIdx::FgPixelIdx => write_generic(context, pv, grp_ctx.fgpxl)?,
        WIPICGraphicsContextIdx::BgPixelIdx => write_generic(context, pv, grp_ctx.bgpxl)?,
        WIPICGraphicsContextIdx::TransPixelIdx => write_generic(context, pv, grp_ctx.transpxl)?,
        WIPICGraphicsContextIdx::AlphaIdx => write_generic(context, pv, grp_ctx.alpha)?,
        WIPICGraphicsContextIdx::PixelopIdx => write_generic(context, pv, grp_ctx.pixel_op_func_ptr)?,
        WIPICGraphicsContextIdx::PixelParam1Idx => write_generic(context, pv, grp_ctx.param1)?,
        WIPICGraphicsContextIdx::FontIdx => write_generic(context, pv, grp_ctx.font)?,
        WIPICGraphicsContextIdx::StyleIdx => write_generic(context, pv, grp_ctx.style)?,
        WIPICGraphicsContextIdx::OffsetIdx => write_generic(context, pv, grp_ctx.offset.map(|value| i32::from(value as i16)))?,
        _ => tracing::warn!("MC_grpGetContext({p_grp_ctx:#x}, {op:?}, {pv:#x}): ignoring invalid op"),
    }

    Ok(())
}

pub async fn put_pixel(context: &mut dyn WIPICContext, dst_fb: WIPICIndirectPtr, x: i32, y: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpPutPixel({:#x}, {x}, {y}, {p_gctx:?})", dst_fb.0);

    let (x, y) = from_context_origin(context, p_gctx, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst_fb)?)?);
    let gctx = read_graphics_context(context, p_gctx)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };
    let clip = with_context_clip(context, p_gctx, clip)?;
    let pixels = draw_through_context(context, &framebuffer, p_gctx, clip, |context, alpha| {
        primitives::put_pixel(context, &framebuffer, x as _, y as _, color, clip, alpha)
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

pub async fn fill_rect(context: &mut dyn WIPICContext, dst_fb: WIPICIndirectPtr, x: i32, y: i32, w: i32, h: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpFillRect({:#x}, {x}, {y}, {w}, {h}, {p_gctx:#x})", dst_fb.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let (x, y) = from_context_origin(context, p_gctx, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst_fb)?)?);
    let gctx = read_graphics_context(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, p_gctx, clip)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    let reach = clip.intersect(&Clip {
        x,
        y,
        width: w as u32,
        height: h as u32,
    });
    let pixels = draw_through_context(context, &framebuffer, p_gctx, reach, |context, alpha| {
        primitives::fill_rect(context, &framebuffer, x, y, w as u32, h as u32, color, clip, alpha)
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    start_angle: i32,
    arc_angle: i32,
    p_gctx: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpDrawArc({:#x}, {x}, {y}, {w}, {h}, {start_angle}, {arc_angle}, {p_gctx:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let (x, y) = from_context_origin(context, p_gctx, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx = read_graphics_context(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, p_gctx, clip)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    let pixels = draw_through_context(context, &framebuffer, p_gctx, clip, |context, alpha| {
        primitives::draw_arc(
            context,
            &framebuffer,
            x,
            y,
            w as u32,
            h as u32,
            start_angle,
            arc_angle,
            color,
            clip,
            alpha,
        )
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

#[allow(clippy::too_many_arguments)]
pub async fn fill_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    start_angle: i32,
    arc_angle: i32,
    p_gctx: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFillArc({:#x}, {x}, {y}, {w}, {h}, {start_angle}, {arc_angle}, {p_gctx:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let (x, y) = from_context_origin(context, p_gctx, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx = read_graphics_context(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, p_gctx, clip)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    let pixels = draw_through_context(context, &framebuffer, p_gctx, clip, |context, alpha| {
        primitives::fill_arc(
            context,
            &framebuffer,
            x,
            y,
            w as u32,
            h as u32,
            start_angle,
            arc_angle,
            color,
            clip,
            alpha,
        )
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

pub async fn create_image(
    context: &mut dyn WIPICContext,
    ptr_image: WIPICWord,
    image_data: WIPICIndirectPtr,
    offset: u32,
    len: u32,
) -> Result<WIPICWord> {
    tracing::debug!("MC_grpCreateImage({ptr_image:#x}, {:#x}, {offset}, {len})", image_data.0);

    let image = create_wipi_image(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<ImageRecord>() as WIPICWord)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(1) // MC_GRP_IMAGE_DONE
}

/// `MC_grpEncodeImage(framebuffer, x, y, w, h, *len)` answers with a region of a framebuffer as BMP
/// bytes in a buffer the caller owns and frees with `MC_knlFree`: the specification says the bytes
/// come from `MC_knlCalloc`. 리얼싸커2009 and 마스터오브소드3 snapshot the screen with it. The region
/// is clipped to the framebuffer, and an empty one answers 0 with a length of 0.
pub async fn encode_image(
    context: &mut dyn WIPICContext,
    framebuffer: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    ptr_len: WIPICWord,
) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpEncodeImage({:#x}, {x}, {y}, {width}, {height}, {ptr_len:#x})", framebuffer.0);

    if ptr_len != 0 {
        write_generic(context, ptr_len, 0u32)?;
    }
    if framebuffer.0 == 0 {
        return Ok(WIPICIndirectPtr(0));
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);
    let (mut x, mut y, mut width, mut height) = (x as i64, y as i64, width as i64, height as i64);
    if x < 0 {
        width += x;
        x = 0;
    }
    if y < 0 {
        height += y;
        y = 0;
    }
    width = width.min(framebuffer.0.width as i64 - x);
    height = height.min(framebuffer.0.height as i64 - y);
    if width <= 0 || height <= 0 {
        return Ok(WIPICIndirectPtr(0));
    }

    let source = framebuffer.image(context)?;
    let encoded = encode_bmp_region(&*source, x as u32, y as u32, width as u32, height as u32)?;

    let memory = context.alloc(encoded.len() as WIPICWord)?;
    context.write_bytes(context.data_ptr(memory)?, &encoded)?;
    if ptr_len != 0 {
        write_generic(context, ptr_len, encoded.len() as u32)?;
    }

    Ok(memory)
}

pub async fn destroy_image(context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyImage({:#x})", image.0);

    let record: ImageRecord = read_generic(context, context.data_ptr(image)?)?;
    for framebuffer in [record.img, record.mask] {
        if framebuffer.0 != 0 {
            let inner: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;
            if inner.buf.0 != 0 {
                context.free(inner.buf)?;
            }
            context.free(framebuffer)?;
        }
    }
    context.free(image)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_image(
    context: &mut dyn WIPICContext,
    framebuffer: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    image: WIPICIndirectPtr,
    sx: i32,
    sy: i32,
    graphics_context: WIPICWord,
) -> Result<()> {
    tracing::debug!(
        "MC_grpDrawImage({:#x}, {dx}, {dy}, {w}, {h}, {:#x}, {sx}, {sy}, {graphics_context:#x})",
        framebuffer.0,
        image.0
    );

    let (dx, dy) = from_context_origin(context, graphics_context, dx, dy)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);
    let image: ImageRecord = read_generic(context, context.data_ptr(image)?)?;

    let src_image = image_with_mask(context, &image)?;
    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, graphics_context, clip)?;

    let reach = clip.intersect(&Clip {
        x: dx,
        y: dy,
        width: w as u32,
        height: h as u32,
    });
    let pixels = draw_through_context(context, &framebuffer, graphics_context, reach, |context, alpha| {
        primitives::draw_image(context, &framebuffer, dx, dy, w as u32, h as u32, &*src_image, sx, sy, clip, alpha)
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

pub async fn flush_lcd(
    context: &mut dyn WIPICContext,
    i: WIPICWord,
    framebuffer: WIPICIndirectPtr,
    x: WIPICWord,
    y: WIPICWord,
    w: WIPICWord,
    h: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFlushLcd({i:#x}, {:#x}, {x:#x}, {y:#x}, {w:#x}, {h:#x})", framebuffer.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);

    let src_canvas = framebuffer.image(context)?;

    let platform = context.system().platform();
    let screen = platform.screen();

    screen.paint(&*src_canvas);

    Ok(())
}

pub async fn get_pixel_from_rgb(_context: &mut dyn WIPICContext, r: i32, g: i32, b: i32) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetPixelFromRGB({r:#x}, {g:#x}, {b:#x})");
    if (r > 0xff) || (g > 0xff) | (b > 0xff) {
        tracing::debug!("MC_grpGetPixelFromRGB({r:#x}, {g:#x}, {b:#x}): value clipped to 8 bits");
    }

    let color = Rgb565Pixel::from_color(Color {
        a: 0xff,
        r: r as u8,
        g: g as u8,
        b: b as u8,
    });

    Ok(color as WIPICWord)
}

pub async fn get_rgb_from_pixel(context: &mut dyn WIPICContext, pixel: i32, r: WIPICWord, g: WIPICWord, b: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_grpGetRGBFromPixel({pixel}, {r:#x}, {g:#x}, {b:#x})");

    let color = Rgb565Pixel::to_color(pixel as u16);

    write_generic(context, r, color.r as i32)?;
    write_generic(context, g, color.g as i32)?;
    write_generic(context, b, color.b as i32)?;

    Ok(pixel)
}

pub async fn get_display_info(context: &mut dyn WIPICContext, reserved: WIPICWord, out_ptr: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetDisplayInfo({reserved:#x}, {out_ptr:#x})");

    assert_eq!(reserved, 0);

    let platform = context.system().platform();
    let screen = platform.screen();

    let info = WIPICDisplayInfo {
        bpp: FRAMEBUFFER_DEPTH,
        depth: 16,
        width: screen.width(),
        height: screen.height(),
        bpl: 2 * screen.width(),
        color_type: 1, // 1==MC_GRP_DIRECT_COLOR_TYPE
        red_mask: 0xf800,
        green_mask: 0x7e0,
        blue_mask: 0x1f,
    };

    write_generic(context, out_ptr, info)?;
    Ok(1)
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_area(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpCopyArea({:#x}, {dx}, {dy}, {w}, {h}, {x}, {y}, {pgc:#x})", dst.0);

    if w < 0 || h < 0 {
        tracing::warn!("Skipping negative dimension");

        return Ok(());
    }
    let (dx, dy) = from_context_origin(context, pgc, dx, dy)?;
    let (x, y) = from_context_origin(context, pgc, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, pgc, clip)?;

    primitives::copy_area(context, &framebuffer, dx, dy, w as u32, h as u32, x, y, clip)
}

pub async fn create_offscreen_framebuffer(context: &mut dyn WIPICContext, w: i32, h: i32) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpCreateOffScreenFrameBuffer({w}, {h})");

    let framebuffer = FrameBuffer::new(context, w as _, h as _, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer.0)?;

    Ok(memory)
}

/// Destroying a null framebuffer or the screen's is nothing to do: the specification says so
/// of the screen, and 컴투스포춘골프3D hands a null one when its five-second wait ends (a
/// dereference here was its "Invalid memory access; address: 4" before any frame). The record's
/// pixel buffer goes back with the record, since `MC_knlGetFreeMemory` answers from what is held.
pub async fn destroy_offscreen_framebuffer(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyOffScreenFrameBuffer({:#x})", framebuffer.0);

    let screen: u32 = read_generic(context, SCREEN_FRAMEBUFFER_PTR)?;
    if framebuffer.0 == 0 || framebuffer.0 == screen {
        return Ok(());
    }

    let record: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;
    if record.buf.0 != 0 {
        context.free(record.buf)?;
    }
    context.free(framebuffer)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_frame_buffer(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    src: WIPICIndirectPtr,
    sx: i32,
    sy: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!(
        "MC_grpCopyFrameBuffer({:#x}, {dx}, {dy}, {w}, {h}, {:#x}, {sx}, {sy}, {pgc:#x})",
        dst.0,
        src.0
    );

    let (dx, dy) = from_context_origin(context, pgc, dx, dy)?;

    let src_framebuffer = FrameBuffer(read_generic(context, context.data_ptr(src)?)?);
    let dst_framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, pgc, clip)?;

    let reach = clip.intersect(&Clip {
        x: dx,
        y: dy,
        width: w as u32,
        height: h as u32,
    });
    let pixels = draw_through_context(context, &dst_framebuffer, pgc, reach, |context, alpha| {
        primitives::copy_framebuffer(
            context,
            &dst_framebuffer,
            dx,
            dy,
            w as u32,
            h as u32,
            &src_framebuffer,
            sx,
            sy,
            clip,
            alpha,
        )
    })?;
    write_operated_pixels(context, &dst_framebuffer, pixels).await
}

pub async fn get_font(_: &mut dyn WIPICContext, face: i32, size: i32, style: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFont({face}, {size}, {style})");

    Ok(0)
}

/// WIPI's default font is 12px tall. `MC_grpGetFontHeight` reports that number and
/// games lay out text lines with it, so we must render at the same height: ab_glyph
/// scales a point size to `pt * 96 / 72` pixels, making 9pt exactly 12px.
const FONT_SIZE: f32 = 9.0;
const FONT_HEIGHT: i32 = 12;
const FONT_ASCENT: i32 = 9;

pub async fn get_font_height(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetFontHeight({font})");

    Ok(FONT_HEIGHT)
}

pub async fn get_font_ascent(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetFontAscent({font})");

    Ok(FONT_ASCENT)
}

pub async fn get_font_descent(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetFontDescent({font})");

    Ok(FONT_HEIGHT - FONT_ASCENT)
}

pub async fn get_string_width(context: &mut dyn WIPICContext, font: i32, ptr_string: WIPICWord, length: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetStringWidth({font}, {ptr_string:#x}, {length})");

    // A length that stops inside a two-byte character measures that character whole. A title that wraps
    // a paragraph by measuring one byte more at a time until it no longer fits breaks the line before the
    // character that way, and through the middle of it when its first byte is taken for anything less.
    let length = if length > 0 && primitives::ends_inside_character(context, ptr_string, length)? {
        length + 1
    } else {
        length
    };
    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(0);
    };
    Ok(string_width(context.system().platform().font(), &string, FONT_SIZE) as i32)
}

pub async fn draw_string(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    ptr_string: WIPICWord,
    length: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpDrawString({:#x}, {x}, {y}, {ptr_string:#x}, {length}, {pgc:#x})", dst.0);

    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(());
    };
    let (x, y) = from_context_origin(context, pgc, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx = read_graphics_context(context, pgc)?;

    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };
    let clip = with_context_clip(context, pgc, clip)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    // KTF's platform takes `y` as the text baseline: 메이플스토리 도적편 (KTF) hardcodes
    // its dialogue at y=293/307 inside a box spanning 273..316 and its speaker name at
    // y=268 on a plate spanning 255..268 — both only fit when glyph ink occupies
    // y-11..y. LGT's platform takes `y` as the cell top instead (see wie-lgt's
    // draw_string), so the conversion lives here rather than in the primitive.
    let top = y + 1 - FONT_HEIGHT;
    let reach = clip.intersect(&Clip {
        y: top,
        height: FONT_HEIGHT as u32,
        ..clip
    });
    let pixels = draw_through_context(context, &framebuffer, pgc, reach, |context, alpha| {
        primitives::draw_text(context, &framebuffer, &string, x, top, color, clip, alpha)
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

/// `MC_grpRepaint` asks the handset to put the LCD's framebuffer on the LCD and to run the
/// title's paint. The Clet runtimes (KTF and LGT) disable the MIDP display's own flush, so what
/// reaches the screen is only what the C side presents: a title that draws with `MC_grp*` into
/// the screen framebuffer and relies on this call to show it (sega미니게임6, a thousand strings a
/// second onto a screen that stayed white) needs the framebuffer presented here. It is presented
/// when the C side drew into it since the last presentation, so a title whose paint draws and
/// flushes on its own (데몬헌터, 바이오크로니클) is not shown a stale frame first.
pub async fn repaint(context: &mut dyn WIPICContext, lcd: i32, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    tracing::debug!("MC_grpRepaint({lcd}, {x}, {y}, {width}, {height})");

    let dirty: u32 = read_generic(context, SCREEN_FRAMEBUFFER_DIRTY_PTR)?;
    tracing::debug!("MC_grpRepaint: screen framebuffer dirty={dirty}");
    if dirty != 0 {
        write_generic(context, SCREEN_FRAMEBUFFER_DIRTY_PTR, 0u32)?;
        let screen: u32 = read_generic(context, SCREEN_FRAMEBUFFER_PTR)?;
        if screen != 0 {
            let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(WIPICIndirectPtr(screen))?)?);
            let frame = framebuffer.image(context)?;
            context.system().platform().screen().paint(&*frame);
        }
    }

    let platform = context.system().platform();
    let screen = platform.screen();
    screen.request_redraw().unwrap();
    context.system().event_queue().push(Event::Repaint);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn get_rgb_pixels(
    context: &mut dyn WIPICContext,
    src: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    pd: WIPICWord,
    ipl: i32,
) -> Result<()> {
    tracing::debug!("MC_grpGetRGBPixels({:#x}, {x}, {y}, {w}, {h}, {pd:#x}, {ipl})", src.0);
    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(src)?)?);
    let image = framebuffer.image(context)?;
    primitives::get_rgb_pixels(context, &*image, x, y, w, h, pd, ipl)
}

#[allow(clippy::too_many_arguments)]
pub async fn set_rgb_pixels(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    psrc: WIPICWord,
    ibpl: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpSetRGBPixels({:#x}, {x}, {y}, {w}, {h}, {psrc:#x}, {ibpl})", dst.0);
    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let (x, y) = from_context_origin(context, pgc, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };
    let clip = with_context_clip(context, pgc, clip)?;
    primitives::set_rgb_pixels(context, &framebuffer, x, y, w, h, psrc, ibpl, clip)
}

pub async fn get_image_framebuffer(context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetImageFrameBuffer({:#x})", image.0);

    let record: ImageRecord = read_generic(context, context.data_ptr(image)?)?;
    Ok(record.img)
}

/// `MC_grpDecodeNextImage(img)` walks an animated source a frame at a time. Every image here is
/// decoded whole when created, so there is no next frame: `MC_GRP_IMAGE_DONE`.
pub async fn decode_next_image(_context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_grpDecodeNextImage({:#x})", image.0);

    Ok(1) // MC_GRP_IMAGE_DONE
}

pub async fn get_image_property(context: &mut dyn WIPICContext, image: WIPICIndirectPtr, property: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetImageProperty({:#x}, {property})", image.0);

    let image: ImageRecord = read_generic(context, context.data_ptr(image)?)?;
    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(image.img)?)?;

    Ok(match property {
        4 => framebuffer.width as _,
        5 => framebuffer.height as _,
        _ => {
            tracing::warn!("unknown property {property}");
            0
        }
    })
}

pub async fn draw_rect(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x: i32, y: i32, w: i32, h: i32, pgc: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpDrawRect({:#x}, {x}, {y}, {w}, {h}, {pgc:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let (x, y) = from_context_origin(context, pgc, x, y)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx = read_graphics_context(context, pgc)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };
    let clip = with_context_clip(context, pgc, clip)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    let pixels = draw_through_context(context, &framebuffer, pgc, clip, |context, alpha| {
        primitives::draw_rect(context, &framebuffer, x, y, w as u32, h as u32, color, clip, alpha)
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

pub async fn draw_line(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x1: i32, y1: i32, x2: i32, y2: i32, pgc: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpDrawLine({:#x}, {x1}, {y1}, {x2}, {y2}, {pgc:#x})", dst.0);

    let (x1, y1) = from_context_origin(context, pgc, x1, y1)?;
    let (x2, y2) = from_context_origin(context, pgc, x2, y2)?;

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx = read_graphics_context(context, pgc)?;
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width as _,
        height: framebuffer.0.height as _,
    };
    let clip = with_context_clip(context, pgc, clip)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    let pixels = draw_through_context(context, &framebuffer, pgc, clip, |context, alpha| {
        primitives::draw_line(context, &framebuffer, x1, y1, x2, y2, color, clip, alpha)
    })?;
    write_operated_pixels(context, &framebuffer, pixels).await
}

pub async fn post_event(context: &mut dyn WIPICContext, id: i32, r#type: i32, param1: i32, param2: i32) -> Result<i32> {
    tracing::debug!("MC_grpPostEvent({id}, {type}, {param1}, {param2})");

    context.system().event_queue().push(Event::Notify { r#type, param1, param2 });

    Ok(0)
}

// it's not documented api, but lgt apps gets pointer via api call
pub async fn get_framebuffer_pointer(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<WIPICWord> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_POINTER({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.buf.0)
}

pub async fn get_framebuffer_width(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_WIDTH({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.width as _)
}

pub async fn get_framebuffer_height(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_HEIGHT({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.height as _)
}

pub async fn get_framebuffer_bpl(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPL({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.bpl as _)
}

pub async fn get_framebuffer_bpp(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPP({:#x})", framebuffer.0);

    // The pixel depth is a fixed screen property on real handsets, so a call here
    // always yields the display depth. Some games rely on that and pass a stale
    // register (e.g. a previous call's return value) as the framebuffer handle. If
    // the handle doesn't resolve to a real framebuffer we must still report the
    // display depth rather than 0 — a 0 makes such a game pick a 32-bit render path
    // and blit 32-bit pixels into our 16-bit framebuffer, corrupting both the image
    // (garbled sprites) and the adjacent heap.
    let bpp = match context.data_ptr(framebuffer) {
        Ok(ptr) => read_generic::<WIPICFramebuffer, _>(context, ptr).map(|fb| fb.bpp).unwrap_or(0),
        Err(_) => 0,
    };

    Ok(if bpp != 0 { bpp as i32 } else { FRAMEBUFFER_DEPTH as i32 })
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};

    use test_utils::TestPlatform;

    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::ByteWrite;

    use crate::{MethodImpl, context::test::TestContext};

    use super::*;

    #[futures_test::test]
    async fn strings_are_positioned_at_the_requested_baseline() -> Result<()> {
        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "test-aid", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        let destination = create_offscreen_framebuffer(&mut context, 32, 32).await?;
        let framebuffer = FrameBuffer(read_generic(&context, context.data_ptr(destination)?)?);
        framebuffer.write(&mut context, &[0; 32 * 32 * 2])?;
        let graphics = context.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
        init_context(&mut context, graphics).await?;
        set_context(&mut context, graphics, WIPICGraphicsContextIdx::FgPixelIdx, 0xffff).await?;
        let text = context.alloc_raw(3)?;
        context.write_bytes(text, &[0xb0, 0xa1, 0])?;
        draw_string(&mut context, destination, 2, 16, text, -1, graphics).await?;

        let bottom = 16 + get_font_descent(&mut context, 0).await?;
        let image = framebuffer.image(&mut context)?;
        // The glyph's cell ends on the baseline row and starts a font height above it; a title that
        // clips a line of text to its own height shows none of it otherwise.
        let top = 16 + 1 - get_font_height(&mut context, 0).await?;
        assert!((0..top).all(|y| (0..32).all(|x| image.get_pixel(x, y).r == 0)));
        assert!((top..=16).any(|y| (0..32).any(|x| image.get_pixel(x, y).r != 0)));
        assert!((bottom..32).all(|y| (0..32).all(|x| image.get_pixel(x, y).r == 0)));

        // A title's operation is asked once about each pixel a glyph touches, and one that answers with what it was
        // given leaves the edges of the glyph as soft as they were.
        async fn keep(_: &mut dyn WIPICContext, source: u32, _: u32, _: u32) -> Result<u32> {
            Ok(source)
        }
        context.functions.push((0x1234, Arc::new(keep.into_body())));
        let plain: Vec<u16> = framebuffer.data(&context, 0..32)?;
        framebuffer.write(&mut context, &[0; 32 * 32 * 2])?;
        set_context(&mut context, graphics, WIPICGraphicsContextIdx::PixelopIdx, 0x1234).await?;
        draw_string(&mut context, destination, 2, 16, text, -1, graphics).await?;
        assert!(plain.iter().any(|pixel| *pixel != 0 && *pixel != 0xffff));
        assert_eq!(framebuffer.data::<u16>(&context, 0..32)?, plain);
        assert_eq!(context.function_calls.len(), plain.iter().filter(|pixel| **pixel != 0).count());

        // Half of the character is as wide as all of it.
        assert_eq!(
            get_string_width(&mut context, 0, text, 1).await?,
            get_string_width(&mut context, 0, text, 2).await?
        );
        Ok(())
    }

    #[futures_test::test]
    async fn encoded_framebuffer_region_can_be_loaded_as_an_image() -> Result<()> {
        let mut context = TestContext::new();
        let source = create_offscreen_framebuffer(&mut context, 3, 2).await?;
        let framebuffer = FrameBuffer(read_generic(&context, context.data_ptr(source)?)?);
        framebuffer.write(&mut context, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0xf8, 0xe0, 0x07])?;
        let length_ptr = context.alloc_raw(4)?;
        let output = context.alloc_raw(4)?;
        for (x, y, width, height) in [(0, 0, 3, 2), (1, 1, 2, 1)] {
            let encoded = encode_image(&mut context, source, x, y, width, height, length_ptr).await?;
            assert_ne!(encoded.0, 0);
            let length: u32 = read_generic(&context, length_ptr)?;
            assert!(length > 0);
            assert_eq!(read_generic::<[u8; 2], _>(&context, context.data_ptr(encoded)?)?, *b"BM");
            create_image(&mut context, output, encoded, 0, length).await?;
            let handle: WIPICIndirectPtr = read_generic(&context, output)?;
            let image: ImageRecord = read_generic(&context, context.data_ptr(handle)?)?;
            let decoded = FrameBuffer(read_generic(&context, context.data_ptr(image.img)?)?).image(&mut context)?;
            assert_eq!((decoded.width(), decoded.height()), (width as u32, height as u32));
            let original = framebuffer.image(&mut context)?;
            for row in 0..height {
                for col in 0..width {
                    let expected = original.get_pixel(x + col, y + row);
                    let actual = decoded.get_pixel(col, row);
                    assert_eq!([actual.r, actual.g, actual.b], [expected.r, expected.g, expected.b]);
                }
            }
        }
        // The region is clipped to the framebuffer; one that leaves nothing answers 0.
        for (x, y, width, height) in [(-1, 0, 1, 1), (0, 0, 0, 1), (3, 0, 1, 1)] {
            assert_eq!(encode_image(&mut context, source, x, y, width, height, length_ptr).await?.0, 0);
            assert_eq!(read_generic::<u32, _>(&context, length_ptr)?, 0);
        }
        Ok(())
    }

    #[futures_test::test]
    async fn indexed_images_preserve_transparency_and_context_clipping() -> Result<()> {
        for (reserved, important, transparent) in [(1u16, 0x101u32, Some(1)), (1, 0x100, Some(0)), (0, 0x101, None), (1, 3, None)] {
            let mut bmp = vec![0u8; 70];
            bmp[..2].copy_from_slice(b"BM");
            bmp[2..6].copy_from_slice(&70u32.to_le_bytes());
            bmp[6..8].copy_from_slice(&reserved.to_le_bytes());
            bmp[10..14].copy_from_slice(&66u32.to_le_bytes());
            bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
            bmp[18..22].copy_from_slice(&3u32.to_le_bytes());
            bmp[22..26].copy_from_slice(&1u32.to_le_bytes());
            bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
            bmp[28..30].copy_from_slice(&8u16.to_le_bytes());
            bmp[34..38].copy_from_slice(&4u32.to_le_bytes());
            bmp[46..50].copy_from_slice(&3u32.to_le_bytes());
            bmp[50..54].copy_from_slice(&important.to_le_bytes());
            // Equal RGB values at distinct indices must retain independent transparency. The colour is
            // not magenta, which mask-less images leave out of a draw regardless of the palette.
            bmp[54..66].copy_from_slice(&[255, 0, 128, 0, 255, 0, 128, 0, 0, 0, 0, 0]);
            bmp[66..70].copy_from_slice(&[0, 1, 2, 0]);

            let mut context = TestContext::new();
            let buffer = context.alloc(bmp.len() as u32)?;
            context.write_bytes(context.data_ptr(buffer)?, &bmp)?;
            let output = context.alloc_raw(4)?;
            create_image(&mut context, output, buffer, 0, bmp.len() as u32).await?;
            let image: WIPICIndirectPtr = read_generic(&context, output)?;
            let destination = create_offscreen_framebuffer(&mut context, 4, 3).await?;
            let framebuffer = FrameBuffer(read_generic(&context, context.data_ptr(destination)?)?);
            let graphics_context = context.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
            init_context(&mut context, graphics_context).await?;
            let clip = context.alloc_raw(16)?;
            write_generic(&mut context, clip, [2i32, 1, 3, 2])?;
            let origin = context.alloc_raw(8)?;
            // A draw is measured from the context's origin and the clip is not, so the same pixels are
            // reached wherever the origin is put.
            for (clip_ptr, origin_x, origin_y) in [(clip, 0, 0), (0, 0, 0), (clip, 3, -2), (0, -1, 1)] {
                set_context(&mut context, graphics_context, WIPICGraphicsContextIdx::ClipIdx, clip_ptr).await?;
                write_generic(&mut context, origin, [origin_x, origin_y])?;
                set_context(&mut context, graphics_context, WIPICGraphicsContextIdx::OffsetIdx, origin).await?;
                framebuffer.write(&mut context, &[0xe0, 0x07].repeat(12))?;
                draw_image(&mut context, destination, 1 - origin_x, 1 - origin_y, 3, 1, image, 0, 0, graphics_context).await?;

                let drawn = framebuffer.image(&mut context)?;
                // The draw reaches the one row it covers and leaves the rows around it alone.
                for (x, y) in [(0, 0), (1, 0), (2, 0), (3, 0), (0, 1), (0, 2), (1, 2), (2, 2), (3, 2)] {
                    let color = drawn.get_pixel(x, y);
                    assert_eq!([color.r, color.g, color.b], [0, 255, 0]);
                }
                for index in 0..3 {
                    let color = drawn.get_pixel(index + 1, 1);
                    let expected = if transparent == Some(index) || (clip_ptr != 0 && index != 1) {
                        [0, 255, 0]
                    } else if index == 2 {
                        [0, 0, 0]
                    } else {
                        [132, 0, 255]
                    };
                    assert_eq!([color.r, color.g, color.b], expected);
                }
            }

            // An alpha given to the context blends the draw over what is there, for as long as the pixel operation
            // that came with it is the context's. The record this context started from has no alpha at all.
            set_context(&mut context, graphics_context, WIPICGraphicsContextIdx::OffsetIdx, 0).await?;
            for (op, value, green) in [
                (WIPICGraphicsContextIdx::AlphaIdx, 128, 120..=136),
                (WIPICGraphicsContextIdx::AlphaIdx, 0, 255..=255),
                (WIPICGraphicsContextIdx::PixelopIdx, 0, 0..=0),
            ] {
                set_context(&mut context, graphics_context, op, value).await?;
                framebuffer.write(&mut context, &[0xe0, 0x07].repeat(12))?;
                draw_image(&mut context, destination, 1, 1, 3, 1, image, 0, 0, graphics_context).await?;

                let color = framebuffer.image(&mut context)?.get_pixel(3, 1);
                assert_eq!([color.r, color.b], [0, 0]);
                assert!(green.contains(&color.g), "{}", color.g);
            }

            // A title's own operation answers for each pixel the draw writes, and for no other.
            async fn recolor(_: &mut dyn WIPICContext, source: u32, original: u32, param1: u32) -> Result<u32> {
                Ok(if source == 0 { original } else { param1 })
            }
            context.functions.push((0x1234, Arc::new(recolor.into_body())));
            set_context(&mut context, graphics_context, WIPICGraphicsContextIdx::PixelopIdx, 0x1234).await?;
            set_context(&mut context, graphics_context, WIPICGraphicsContextIdx::PixelParam1Idx, 0x001f).await?;
            framebuffer.write(&mut context, &[0xe0, 0x07].repeat(12))?;
            draw_image(&mut context, destination, 1, 1, 3, 1, image, 0, 0, graphics_context).await?;

            let drawn = framebuffer.image(&mut context)?;
            for index in 0..3 {
                let color = drawn.get_pixel(index + 1, 1);
                let expected = if transparent == Some(index) || index == 2 {
                    [0, 255, 0]
                } else {
                    [0, 0, 255]
                };
                assert_eq!([color.r, color.g, color.b], expected);
            }
            assert_eq!(context.function_calls.len(), if transparent.is_some() { 2 } else { 3 });
            assert!(context.function_calls.iter().all(|(_, arguments)| arguments[1..] == [0x07e0, 0x001f]));
        }
        Ok(())
    }

    #[futures_test::test]
    async fn context_values_are_written_to_the_output_pointer() -> Result<()> {
        // A title may read the record itself, so the fields sit where each carrier's headers put them:
        // 크로이센 takes its text and sprite opacity from the word at 0x18 in its LGT build and from the
        // one at 0x20 in its KTF build.
        for (layout, foreground, alpha) in [(GraphicsContextLayout::Narrow, 0x0c, 0x18), (GraphicsContextLayout::Wide, 0x14, 0x20)] {
            let mut context = TestContext::new().with_graphics_context_layout(layout);
            let ptr_context = context.alloc_raw(0x40)?;
            let input = context.alloc_raw(16)?;
            let output = context.alloc_raw(20)?;
            init_context(&mut context, ptr_context).await?;

            let set = set_context.into_body();
            let get = get_context.into_body();
            for (op, value) in [
                (1, 0x12345678),
                (2, 0x87654321),
                (3, 0xff00ff),
                (4, 128),
                (5, 0x1001),
                (6, 42),
                (7, 8),
                (8, 1),
            ] {
                set.call(&mut context, Box::new([ptr_context, op, value])).await?;
                write_generic(&mut context, output, [0xccccccccu32; 5])?;
                get.call(&mut context, Box::new([ptr_context, op, output])).await?;
                assert_eq!(
                    read_generic::<[u32; 5], _>(&context, output)?,
                    [value, 0xcccccccc, 0xcccccccc, 0xcccccccc, 0xcccccccc]
                );
            }

            write_generic(&mut context, input, [-5i32, -8, 176, 220])?;
            set.call(&mut context, Box::new([ptr_context, 0, input])).await?;
            write_generic(&mut context, output, [999i32; 5])?;
            get.call(&mut context, Box::new([ptr_context, 0, output])).await?;
            assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-5, -8, 176, 220, 999]);

            assert_eq!(read_generic::<u32, _>(&context, ptr_context)? & 1, 1);
            set.call(&mut context, Box::new([ptr_context, 0, 0])).await?;
            assert_eq!(read_generic::<u32, _>(&context, ptr_context)? & 1, 0);
            assert_eq!(read_generic::<u32, _>(&context, ptr_context + foreground)?, 0x12345678);
            assert_eq!(read_generic::<u32, _>(&context, ptr_context + alpha)?, 128);

            write_generic(&mut context, input, [-12i32, 34])?;
            set.call(&mut context, Box::new([ptr_context, 10, input])).await?;
            write_generic(&mut context, output, [999i32; 5])?;
            get.call(&mut context, Box::new([ptr_context, 10, output])).await?;
            assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-12, 34, 999, 999, 999]);

            get.call(&mut context, Box::new([ptr_context, 0xff, output])).await?;
            assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-12, 34, 999, 999, 999]);
        }
        Ok(())
    }
}
