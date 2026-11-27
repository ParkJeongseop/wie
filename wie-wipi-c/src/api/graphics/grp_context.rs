use core::mem;

use bytemuck::{Pod, Zeroable};
use wipi_types::wipic::{WIPICGraphicsContext, WIPICWord};

use wie_util::{Result, read_generic, write_generic};

use crate::{WIPICContext, method::ParamConverter};

/// How a carrier lays out `MC_GrpContext`. A title is free to read the record it owns instead of asking
/// `MC_grpGetContext`, so each carrier's fields have to sit where its own headers put them.
#[derive(Clone, Copy)]
pub enum GraphicsContextLayout {
    /// The clip and the offset are 16-bit coordinates.
    Narrow,
    /// The clip and the offset are `M_Int32` coordinates, which moves every field after them.
    Wide,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WideGraphicsContext {
    mask: u32,
    clip: [i32; 4],
    fgpxl: u32,
    bgpxl: u32,
    transpxl: u32,
    alpha: u32,
    offset: [i32; 2],
    pixel_op_func_ptr: WIPICWord,
    param1: WIPICWord,
    reserved: WIPICWord,
    font: WIPICWord,
    style: WIPICWord,
}

pub fn read_graphics_context(context: &dyn WIPICContext, address: WIPICWord) -> Result<WIPICGraphicsContext> {
    match context.graphics_context_layout() {
        GraphicsContextLayout::Narrow => read_generic(context, address),
        GraphicsContextLayout::Wide => {
            let wide: WideGraphicsContext = read_generic(context, address)?;

            Ok(WIPICGraphicsContext {
                mask: wide.mask,
                clip: wide.clip.map(|value| value as u16),
                fgpxl: wide.fgpxl,
                bgpxl: wide.bgpxl,
                transpxl: wide.transpxl,
                alpha: wide.alpha,
                offset: wide.offset.map(|value| value as u16),
                pixel_op_func_ptr: wide.pixel_op_func_ptr,
                param1: wide.param1,
                reserved: wide.reserved,
                font: wide.font,
                style: wide.style,
            })
        }
    }
}

pub fn write_graphics_context(context: &mut dyn WIPICContext, address: WIPICWord, graphics_context: WIPICGraphicsContext) -> Result<()> {
    match context.graphics_context_layout() {
        GraphicsContextLayout::Narrow => write_generic(context, address, graphics_context),
        GraphicsContextLayout::Wide => {
            let wide = WideGraphicsContext {
                mask: graphics_context.mask,
                clip: graphics_context.clip.map(|value| i32::from(value as i16)),
                fgpxl: graphics_context.fgpxl,
                bgpxl: graphics_context.bgpxl,
                transpxl: graphics_context.transpxl,
                alpha: graphics_context.alpha,
                offset: graphics_context.offset.map(|value| i32::from(value as i16)),
                pixel_op_func_ptr: graphics_context.pixel_op_func_ptr,
                param1: graphics_context.param1,
                reserved: graphics_context.reserved,
                font: graphics_context.font,
                style: graphics_context.style,
            };

            write_generic(context, address, wide)
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum WIPICGraphicsContextIdx {
    ClipIdx = 0,
    FgPixelIdx = 1,
    BgPixelIdx = 2,
    TransPixelIdx = 3,
    AlphaIdx = 4,
    PixelopIdx = 5,
    PixelParam1Idx = 6,
    FontIdx = 7,
    StyleIdx = 8,
    XorModeIdx = 9,
    OffsetIdx = 10,
    OutlineIdx = 11,

    /// Unknown values are mapped to this enum value.
    /// Note that this field doesn't exist in WIPI and the choice of this ordinal is arbitrary.
    Invalid = 0xff,
}

impl ParamConverter<WIPICGraphicsContextIdx> for WIPICGraphicsContextIdx {
    fn convert(_context: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICGraphicsContextIdx {
        if raw >= (Self::ClipIdx as WIPICWord) && raw <= (Self::OutlineIdx as WIPICWord) {
            // SAFETY: WIPICGraphicsContextIdx has CWord repr and is unit only.
            let x: Self = unsafe { mem::transmute(raw) };
            x
        } else {
            Self::Invalid
        }
    }
}
