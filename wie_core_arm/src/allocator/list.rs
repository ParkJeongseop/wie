use alloc::{format, vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_util::{ByteRead, Result, WieError, read_generic, write_generic};

use crate::core::ArmCore;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct ListAllocationHeader {
    data: u32,
}

impl ListAllocationHeader {
    pub fn new(size: u32, in_use: bool) -> Self {
        Self {
            data: size | ((in_use as u32) << 31),
        }
    }

    pub fn size(&self) -> u32 {
        self.data & 0x7FFFFFFF
    }

    pub fn in_use(&self) -> bool {
        self.data & 0x80000000 != 0
    }
}

const CANARY_SIZE: u32 = 4;
const CANARY_VALUE: u32 = 0xDEADBEEF;

const HEADER_SIZE: u32 = size_of::<ListAllocationHeader>() as u32;

// Every well-formed block size is a multiple of 4 (a 4-byte header plus 4-aligned
// payload plus a 4-byte canary), non-zero, and stays within the heap. A header that
// violates this has been scribbled over by a guest buffer overflow.
fn is_plausible_size(cursor: u32, size: u32, end: u32) -> bool {
    size >= HEADER_SIZE && size.is_multiple_of(4) && cursor.checked_add(size).is_some_and(|block_end| block_end <= end)
}

pub struct ListAllocator;

impl ListAllocator {
    pub fn init(core: &mut ArmCore, base_address: u32, size: u32) -> Result<()> {
        let header = ListAllocationHeader::new(size, false);

        write_generic(core, base_address, header)?;

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, base_address: u32, base_size: u32, size: u32) -> Result<u32> {
        let size_to_alloc = (size as usize + size_of::<ListAllocationHeader>()).next_multiple_of(4) as u32 + CANARY_SIZE;

        let address = Self::find_address(core, base_address, base_size, size_to_alloc)?;

        let previous_header: ListAllocationHeader = read_generic(core, address)?;

        let header = ListAllocationHeader::new(size_to_alloc, true);
        write_generic(core, address, header)?;

        // write next
        if previous_header.size() > size_to_alloc {
            let next_header = ListAllocationHeader::new(previous_header.size() - size_to_alloc, false);
            write_generic(core, address + size_to_alloc, next_header)?;
        }

        // write canary
        write_generic(core, address + size_to_alloc - CANARY_SIZE, CANARY_VALUE)?;

        tracing::trace!("Allocated {size:#x} bytes at {:#x}", address + size_of::<ListAllocationHeader>() as u32);

        Ok(address + size_of::<ListAllocationHeader>() as u32)
    }

    pub fn free(core: &mut ArmCore, address: u32) -> Result<()> {
        let base_address = address - size_of::<ListAllocationHeader>() as u32;

        tracing::trace!("Freeing {address:#x}");

        let header: ListAllocationHeader = read_generic(core, base_address)?;
        if !header.in_use() {
            // Freeing an already-free block is harmless; buggy apps do this and
            // ran fine on real handsets, so tolerate it instead of aborting.
            tracing::warn!("Double free at {address:#x}");
            return Ok(());
        }

        let canary_value: u32 = read_generic(core, base_address + header.size() - CANARY_SIZE)?;
        if canary_value != CANARY_VALUE {
            return Err(WieError::FatalError(format!(
                "Invalid canary value at {base_address:#x}: expected {CANARY_VALUE:#x}, got {canary_value:#x}"
            )));
        }

        let header = ListAllocationHeader::new(header.size(), false);
        write_generic(core, base_address, header)?;

        Ok(())
    }

    fn find_address(core: &mut ArmCore, base_address: u32, base_size: u32, size: u32) -> Result<u32> {
        let end = base_address + base_size;
        let mut cursor = base_address;
        loop {
            let mut header: ListAllocationHeader = read_generic(core, cursor)?;

            if !is_plausible_size(cursor, header.size(), end) {
                // A guest buffer overflow has scribbled over this block header: a game blit
                // running a few bytes past its own buffer clobbers the following block's
                // header with raw RGB565 pixels. The pixel bytes land in both the size and
                // the in-use bit, so neither field can be trusted here — the in-use bit is
                // just whatever the pixel colour happened to be. We rebuild the block as
                // free: its true extent is the span up to the next in-use block (found by a
                // canary-validated scan) or the heap end, so reconstruct that, repair the
                // header, and let the walk resynchronize instead of derailing on a bogus
                // size. The canary scan preserves any genuine in-use block after the
                // corruption; the block *at* the corrupt header is unrecoverable either way
                // (the old walk would have crashed on it), so treating it as free space is
                // strictly more resilient.
                let boundary = Self::find_next_inuse_boundary(core, cursor + HEADER_SIZE, end)?;
                let repaired = ListAllocationHeader::new(boundary - cursor, false);
                write_generic(core, cursor, repaired)?;
                tracing::warn!(
                    "Repaired corrupt block header at {cursor:#x} (was {:#x}); reclaimed {:#x} bytes up to {boundary:#x}",
                    header.data,
                    boundary - cursor
                );
                header = repaired;
            }

            if !header.in_use() {
                loop {
                    let next = cursor + header.size();
                    if next >= end {
                        break;
                    }
                    let next_header: ListAllocationHeader = read_generic(core, next)?;
                    if next_header.in_use() || !is_plausible_size(next, next_header.size(), end) {
                        break;
                    }
                    header = ListAllocationHeader::new(header.size() + next_header.size(), false);
                    write_generic(core, cursor, header)?;
                }
                if header.size() >= size {
                    return Ok(cursor);
                }
            }

            cursor += header.size();
            if cursor >= end {
                break;
            }
        }

        Err(WieError::AllocationFailure)
    }

    // Scan forward from `start` for the first structurally valid in-use block header,
    // returning its address, or `end` if none is found. Used to recover the true right
    // edge of a free block whose header a guest overflow corrupted. A block is accepted
    // only when its in-use bit is set, its size is well-formed, and its trailing canary
    // matches — the 32-bit canary makes false positives astronomically unlikely, and a
    // false positive would only shrink the reclaimed span (safe), never grow it.
    fn find_next_inuse_boundary(core: &mut ArmCore, start: u32, end: u32) -> Result<u32> {
        const CHUNK: usize = 64 * 1024;

        let mut buf = vec![0u8; CHUNK];
        let mut base = start & !3;
        while base + HEADER_SIZE <= end {
            let want = ((end - base) as usize).min(CHUNK);
            let read = core.read_bytes(base, &mut buf[..want])?;
            let scanned = read & !3;
            if scanned == 0 {
                break;
            }

            let mut off = 0;
            while off + 4 <= scanned {
                let data = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
                if data & 0x80000000 != 0 {
                    let size = data & 0x7FFFFFFF;
                    let p = base + off as u32;
                    if size >= HEADER_SIZE + CANARY_SIZE && is_plausible_size(p, size, end) {
                        let canary: u32 = read_generic(core, p + size - CANARY_SIZE)?;
                        if canary == CANARY_VALUE {
                            return Ok(p);
                        }
                    }
                }
                off += 4;
            }

            base += scanned as u32;
        }

        Ok(end)
    }
}

#[cfg(test)]
mod tests {
    use wie_util::{Result, WieError, write_generic};

    use crate::ArmCore;

    use super::ListAllocator;

    #[test]
    fn test_allocator() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        assert_eq!(address, 0x40000004);

        Ok(())
    }

    #[test]
    fn test_coalesce_adjacent_free_blocks() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x400)?;
        let a = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x100)?;
        let b = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x100)?;
        let _c = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x1e8)?;

        ListAllocator::free(&mut core, a)?;
        ListAllocator::free(&mut core, b)?;

        let merged = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x150)?;
        assert_eq!(merged, a);

        Ok(())
    }

    #[test]
    fn test_double_free_is_tolerated() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        ListAllocator::free(&mut core, address)?;
        // Buggy apps double-free and ran fine on real handsets, so this is tolerated
        // rather than fatal (see ListAllocator::free).
        let result = ListAllocator::free(&mut core, address);

        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_reconstruct_corrupt_free_tail() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x100000)?;

        let base = 0x40000000;
        let region = 0x100000;
        ListAllocator::init(&mut core, base, region)?;

        // one live block, then the free-tail remainder header directly behind it
        let a = ListAllocator::alloc(&mut core, base, region, 0x1000)?;
        let tail = (a - 4) + 0x1008; // block a occupies size_to_alloc(0x1000) = 0x1008 bytes

        // guest RGB565 overflow scribbles a white pixel over the tail header:
        // 0x0000ffff -> in_use=0, size=0xffff which is 4-misaligned
        write_generic(&mut core, tail, 0x0000ffffu32)?;

        // a large allocation must still succeed by reconstructing the tail
        let big = ListAllocator::alloc(&mut core, base, region, 0x8000)?;
        assert_eq!(big, tail + 4);

        Ok(())
    }

    #[test]
    fn test_reconstruct_corrupt_header_with_in_use_bit() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x100000)?;

        let base = 0x40000000;
        let region = 0x100000;
        ListAllocator::init(&mut core, base, region)?;

        let a = ListAllocator::alloc(&mut core, base, region, 0x1000)?;
        let tail = (a - 4) + 0x1008;

        // a magenta RGB565 pixel (0xf81f) scribbled over the tail header: size 0xf81ff81f is
        // 4-misaligned AND its top bit is set, so it must NOT be mistaken for a live block.
        write_generic(&mut core, tail, 0xf81ff81fu32)?;

        let big = ListAllocator::alloc(&mut core, base, region, 0x8000)?;
        assert_eq!(big, tail + 4);

        Ok(())
    }

    #[test]
    fn test_reconstruct_stops_at_inuse_block() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x100000)?;

        let base = 0x40000000;
        let region = 0x100000;
        ListAllocator::init(&mut core, base, region)?;

        let a = ListAllocator::alloc(&mut core, base, region, 0x1000)?;
        let b = ListAllocator::alloc(&mut core, base, region, 0x1000)?;
        ListAllocator::free(&mut core, a)?;

        // corrupt the now-free hole where `a` used to be
        write_generic(&mut core, a - 4, 0x0000ffffu32)?;

        // a request larger than the hole must reconstruct only up to the in-use block `b`,
        // never handing out `b`'s live memory: the allocation lands past `b` in the tail
        let big = ListAllocator::alloc(&mut core, base, region, 0x8000)?;
        let b_end = (b - 4) + 0x1008;
        assert!(big - 4 >= b_end, "allocation overlapped the in-use block b");

        Ok(())
    }

    #[test]
    fn test_corrupted_canary_returns_error() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        write_generic(&mut core, address + 4, 0u32)?;
        let result = ListAllocator::free(&mut core, address);

        assert!(matches!(result, Err(WieError::FatalError(_))));

        Ok(())
    }
}
