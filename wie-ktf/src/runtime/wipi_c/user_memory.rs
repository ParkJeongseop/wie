use wipi_types::wipic::WIPICWord;

use wie_util::{Result, read_generic, write_generic};
use wie_wipi_c::WIPICContext;

// The user-memory library `MC_knlGetDLLInterface("MXUserMemInterf")` hands out: a title gives it a
// buffer of its own and cuts that buffer into blocks, so the library never owns memory. Its
// bookkeeping lives in the buffer: the first word is the buffer's size and the second the offset of
// the next free byte, and blocks are cut from the rest in order. Blocks are not handed out again
// after a free; the one caller (액션히어로3D) allocates its set once at startup.

const HEADER_SIZE: u32 = 8;

/// `create(pool, size)`.
pub async fn create(context: &mut dyn WIPICContext, pool: WIPICWord, size: WIPICWord) -> Result<i32> {
    tracing::debug!("MXUserMem create({pool:#x}, {size:#x})");

    write_generic(context, pool, size)?;
    write_generic(context, pool + 4, HEADER_SIZE)?;

    Ok(0)
}

/// `alloc(pool, size)`: a block of the pool, or null when it is full.
pub async fn alloc(context: &mut dyn WIPICContext, pool: WIPICWord, size: WIPICWord) -> Result<WIPICWord> {
    let total: u32 = read_generic(context, pool)?;
    let next: u32 = read_generic(context, pool + 4)?;
    let size = size.saturating_add(3) & !3;

    let block = match next.checked_add(size) {
        Some(end) if end <= total => {
            write_generic(context, pool + 4, end)?;
            pool + next
        }
        _ => 0,
    };
    tracing::debug!("MXUserMem alloc({pool:#x}, {size:#x}) = {block:#x}");

    Ok(block)
}

/// `free(pool, block)`.
pub async fn free(_context: &mut dyn WIPICContext, pool: WIPICWord, block: WIPICWord) -> Result<i32> {
    tracing::debug!("MXUserMem free({pool:#x}, {block:#x})");

    Ok(0)
}
