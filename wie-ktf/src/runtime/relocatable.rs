//! The older generation of KTF client images is not code at its first byte. A few titles
//! (테일즈판타지, 텐가이, 영웅전설3) ship one, and the normal loader — which runs the image at
//! `IMAGE_BASE + 1` to let it rebase itself — faults a few instructions in, because what it ran as
//! Thumb was the image header.
//!
//! What they carry instead is a relocatable module:
//!
//! ```text
//! u32 bss_size            what the file name's decimal suffix also names
//! u32 relocation_count
//! u32 relocations[]       segment offsets of words to rebase, the first one zero
//! ...segment...
//! ```
//!
//! and the segment opens with eleven words before any code:
//!
//! ```text
//! seg[0]     the module descriptor (its class table)
//! seg[1]     zero
//! seg[2]     the end of the writable tail
//! seg[3..4]  the read-only data
//! seg[5]     a thunk
//! seg[6]     the static base, where the module's own pointer table starts
//! seg[7]     the end of the bytes the file carries
//! seg[8]     0x13580001                     a marker
//! seg[9]     the module entry, Thumb (odd)
//! seg[10]    0x13580001                     a marker
//! seg[11..]  code
//! ```
//!
//! Every relocated word gets the segment's load address added. The header stays mapped in front of
//! the segment, because the segment's own offsets are relative to it, which is why the two markers
//! around the entry — plus the first relocation being zero and a relocation count that fits the
//! file — are enough to tell this format from a normal image (whose first bytes, read this way,
//! give a relocation count far larger than the file).
//!
//! This module only recognises and relocates the image; linking the classes it publishes and
//! running its entry under the platform is the loader's job.

use alloc::vec::Vec;

use wie_util::{Result, WieError};

/// The fixed part of the header: the BSS size and the relocation count, then the relocation table.
const HEADER_WORDS: usize = 2;
/// Where the segment's own header keeps the module entry, flanked by [`MARKER`] on both sides.
const ENTRY_OFFSET: usize = 0x24;
/// The constant at the two words either side of the entry; what makes a header certain.
const MARKER: u32 = 0x13580001;
/// The segment-header word holding the static base the module's position-independent code uses.
const STATIC_BASE_OFFSET: usize = 0x18;
/// The segment-header word holding the end of the bytes the file carries.
const SEGMENT_END_OFFSET: usize = 0x1c;

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(offset..offset + 4)?.try_into().ok()?))
}

/// A recognised relocatable module: where its segment begins in the file and the segment offsets of
/// the words to rebase.
pub struct RelocatableClient {
    segment_start: usize,
    relocations: Vec<u32>,
}

impl RelocatableClient {
    /// Reports whether an image is one of the older relocatable modules and, if so, where its
    /// segment and relocations are. Anything that does not fit exactly is not one, because the
    /// alternative reading — a Thumb entry at the first byte — is what every other image carries.
    pub fn parse(data: &[u8]) -> Option<Self> {
        // The header plus one relocation is the least this can be; the first relocation is read below.
        if data.len() < (HEADER_WORDS + 1) * 4 {
            return None;
        }
        let count = read_u32(data, 4)? as usize;
        // The first relocation rebases the segment's own first word, so it is zero in every module.
        // It is the one word of the table whose value the format fixes, and reading the table one
        // word out is exactly the mistake this layout invites.
        if read_u32(data, 8)? != 0 {
            return None;
        }
        if count == 0 || count > data.len() / 4 {
            return None;
        }
        let segment_start = HEADER_WORDS * 4 + count * 4;
        let segment = data.get(segment_start..)?;
        if segment.len() < ENTRY_OFFSET + 8 {
            return None;
        }
        if read_u32(segment, ENTRY_OFFSET - 4)? != MARKER || read_u32(segment, ENTRY_OFFSET + 4)? != MARKER {
            return None;
        }

        let relocations = (0..count)
            .map(|index| read_u32(data, HEADER_WORDS * 4 + index * 4).unwrap_or(0))
            .collect();
        Some(Self { segment_start, relocations })
    }

    /// Rebases a copy of the whole file for a load at `base`. The header stays in front of the
    /// segment so that the segment lands where its own offsets say, so what moves is only what the
    /// relocation table names and the pointer table the segment header bounds — both by the
    /// segment's load address. A relocation that names a word outside the segment is refused rather
    /// than skipped: one entry that does not fit means the reading of the header is wrong.
    pub fn relocate(&self, data: &[u8], base: u32) -> Result<Vec<u8>> {
        let mut image = data.to_vec();
        let addend = base.wrapping_add(self.segment_start as u32);
        let segment_len = image.len() - self.segment_start;

        for (index, &offset) in self.relocations.iter().enumerate() {
            let offset = offset as usize;
            if offset + 4 > segment_len {
                return Err(WieError::FatalError(alloc::format!(
                    "KTF relocation {index} names offset {offset:#x} outside the {segment_len:#x}-byte segment"
                )));
            }
            let at = self.segment_start + offset;
            let value = u32::from_le_bytes(image[at..at + 4].try_into().unwrap());
            image[at..at + 4].copy_from_slice(&value.wrapping_add(addend).to_le_bytes());
        }

        // The module's own pointer table sits between the static base and the segment end, and every
        // word of it is a segment offset that has to move too. Both bounds are read from the
        // original file, because by now the copy holds their rebased values while the table's
        // offsets are still the segment's.
        let static_base = read_u32(data, self.segment_start + STATIC_BASE_OFFSET).unwrap_or(0) as usize;
        let segment_end = read_u32(data, self.segment_start + SEGMENT_END_OFFSET).unwrap_or(0) as usize;
        if !static_base.is_multiple_of(4) || !segment_end.is_multiple_of(4) || static_base > segment_end || segment_end > segment_len {
            return Err(WieError::FatalError(alloc::format!(
                "KTF static base {static_base:#x} and segment end {segment_end:#x} do not bound a table in a {segment_len:#x}-byte segment"
            )));
        }
        let mut offset = static_base;
        while offset + 4 <= segment_end {
            let at = self.segment_start + offset;
            let value = u32::from_le_bytes(image[at..at + 4].try_into().unwrap());
            image[at..at + 4].copy_from_slice(&value.wrapping_add(addend).to_le_bytes());
            offset += 4;
        }

        Ok(image)
    }

    /// The module entry, read out of a relocated image (Thumb, so the address is odd).
    pub fn entry_address(&self, image: &[u8]) -> u32 {
        read_u32(image, self.segment_start + ENTRY_OFFSET).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{MARKER, RelocatableClient};
    use alloc::vec::Vec;

    /// Builds a minimal valid module: two relocations (the mandatory zero and one that rebases the
    /// entry word), a segment whose header carries the markers around the entry and a one-word
    /// pointer table the static base/segment end bound.
    fn sample() -> Vec<u8> {
        let relocations: [u32; 2] = [0, ENTRY];
        const ENTRY: u32 = 0x24;
        let mut segment = [0u32; 16];
        segment[0] = 0x100; // descriptor pointer (rebased by relocation 0)
        segment[6] = 0x30; // static base: the pointer table starts here
        segment[7] = 0x34; // segment end: one word of table (at 0x30)
        segment[8] = MARKER;
        segment[9] = 0x51; // entry offset, odd (Thumb); rebased by relocation 1 (offset 0x24)
        segment[10] = MARKER;
        segment[12] = 0x200; // the one pointer-table word (at offset 0x30)

        let mut data = Vec::new();
        data.extend_from_slice(&64u32.to_le_bytes()); // bss size
        data.extend_from_slice(&(relocations.len() as u32).to_le_bytes());
        for r in relocations {
            data.extend_from_slice(&r.to_le_bytes());
        }
        for w in segment {
            data.extend_from_slice(&w.to_le_bytes());
        }
        data
    }

    #[test]
    fn parses_a_relocatable_module_and_rejects_a_normal_image() {
        let module = sample();
        assert!(RelocatableClient::parse(&module).is_some());

        // A normal image: first bytes are Thumb code, so the "relocation count" is huge.
        let normal = [0xf0u8, 0xb5, 0x00, 0xaf, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        assert!(RelocatableClient::parse(&normal).is_none());

        // A first relocation that is not zero is rejected.
        let mut broken = module.clone();
        broken[8] = 1;
        assert!(RelocatableClient::parse(&broken).is_none());
    }

    #[test]
    fn relocation_adds_the_segment_load_address_to_named_words_and_the_pointer_table() {
        let module = sample();
        let client = RelocatableClient::parse(&module).unwrap();
        let segment_start = (2 + 2) * 4; // header + two relocations
        let base = 0x100000u32;
        let addend = base + segment_start as u32;

        let image = client.relocate(&module, base).unwrap();
        let word = |off: usize| u32::from_le_bytes(image[segment_start + off..segment_start + off + 4].try_into().unwrap());

        assert_eq!(word(0), 0x100 + addend, "descriptor pointer rebased by relocation 0");
        assert_eq!(word(0x24), 0x51 + addend, "entry rebased by relocation 1");
        assert_eq!(word(0x30), 0x200 + addend, "pointer-table word rebased");
        assert_eq!(word(0x20), MARKER, "marker untouched");
        assert_eq!(client.entry_address(&image), 0x51 + addend);
    }
}
