use alloc::{format, string::String, vec::Vec};
use core::fmt::Display;

use wie_core_arm::ArmCore;
use wie_util::{ByteRead, WieError, read_generic, read_null_terminated_string_bytes};

use super::Result;

#[derive(Clone)]
pub struct JavaFullName {
    pub tag: u8,
    value: String,
    name_offset: usize,
}

impl JavaFullName {
    /// KTF stores a 1-byte tag ahead of the "descriptor+name" text: the low byte of
    /// the Java string hash (h*31+c) over it. client.bin scans member tables by this
    /// tag, so writing 0 lets tag-only lookups land on the wrong overload (놈3 jumped
    /// to <init>([CII)V instead of <init>(Ljava/lang/String;)V).
    pub fn new(name: &str, descriptor: &str) -> Self {
        let value = format!("{descriptor}+{name}");
        let tag = value.bytes().fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32)) as u8;

        Self {
            tag,
            value,
            name_offset: descriptor.len() + 1,
        }
    }

    pub fn from_ptr(core: &ArmCore, ptr: u32) -> Result<Self> {
        let tag = read_generic(core, ptr)?;

        let value = read_null_terminated_string_bytes(core, ptr + 1)?;
        let value = String::from_utf8(value).unwrap();
        let name_offset = value.find('+').unwrap() + 1;

        Ok(Self { tag, value, name_offset })
    }

    pub fn matches(core: &ArmCore, ptr: u32, name: &str, descriptor: &str) -> Result<bool> {
        let expected = Self::text_parts(name, descriptor);
        let length = descriptor.len() + name.len() + 2;
        let last = name.as_bytes().last().copied().unwrap_or(b'+');
        let mut compared = 0;
        let mut buffer = [0; 32];
        while compared != length {
            let cursor = ptr.wrapping_add(1 + compared as u32);
            let size = (length - compared).min(buffer.len());
            let (size, read) = match core.read_bytes(cursor, &mut buffer[..size]) {
                // A shorter non-matching name may end at the memory boundary.
                Err(WieError::InvalidMemoryAccess(_)) => (1, core.read_bytes(cursor, &mut buffer[..1])?),
                result => (size, result?),
            };
            if read != size {
                return Err(WieError::FatalError(format!("Short read at {cursor:#x}: expected {size}, got {read}")));
            }
            // A name of another length is told apart by where it ends, and most of the same length by their
            // last letter, before the text is compared.
            let ends = compared + size != length || (buffer[size - 1] == 0 && (size == 1 || buffer[size - 2] == last));
            if !ends || !Self::text_continues(&buffer[..size], compared, expected) {
                return Ok(false);
            }
            compared += size;
        }
        Ok(true)
    }

    /// Which of `names`, given as (name, descriptor), the member name at `ptr` is. The name is read once for
    /// all of them unless it is longer than what is read or ends at the memory boundary.
    pub fn position(core: &ArmCore, ptr: u32, names: &[(&str, &str)]) -> Result<Option<usize>> {
        let mut buffer = [0; 32];
        let read = match core.read_bytes(ptr.wrapping_add(1), &mut buffer) {
            Err(WieError::InvalidMemoryAccess(_)) => 0,
            result => result?,
        };
        let end = buffer[..read].iter().position(|&byte| byte == 0);
        for (index, (name, descriptor)) in names.iter().enumerate() {
            let length = descriptor.len() + name.len() + 2;
            // A name of another length is told apart by where it ends, and most of the same length by their
            // last letter, before the text is compared.
            let last = name.as_bytes().last().copied().unwrap_or(b'+');
            if end.map_or(length <= read, |end| end + 1 != length || buffer[end - 1] != last) {
                continue;
            }
            let head = &buffer[..read.min(length)];
            if Self::text_continues(head, 0, Self::text_parts(name, descriptor))
                && (head.len() == length || Self::matches(core, ptr, name, descriptor)?)
            {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// The text a member carries after its tag, in the pieces it is compared by.
    fn text_parts<'a>(name: &'a str, descriptor: &'a str) -> [&'a [u8]; 4] {
        [descriptor.as_bytes(), b"+", name.as_bytes(), b"\0"]
    }

    /// Whether `bytes` are what the text `expected` gives in pieces holds from its byte `skipped` on.
    fn text_continues(mut bytes: &[u8], mut skipped: usize, expected: [&[u8]; 4]) -> bool {
        for part in expected {
            let rest = part.get(skipped..).unwrap_or_default();
            skipped = skipped.saturating_sub(part.len());
            let (head, tail) = bytes.split_at(rest.len().min(bytes.len()));
            // These are a few bytes each; comparing them in place is cheaper than a call.
            if head.iter().zip(rest).any(|(read, expected)| read != expected) {
                return false;
            }
            bytes = tail;
        }
        true
    }

    pub fn name(&self) -> &str {
        &self.value[self.name_offset..]
    }

    pub fn descriptor(&self) -> &str {
        &self.value[..self.name_offset - 1]
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.value.len() + 2);

        bytes.push(self.tag);
        bytes.extend_from_slice(self.value.as_bytes());
        bytes.push(0);

        bytes
    }
}

impl Display for JavaFullName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name().fmt(f)?;
        self.descriptor().fmt(f)?;
        write!(f, "@{}", self.tag)?;

        Ok(())
    }
}

impl PartialEq for JavaFullName {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor() == other.descriptor() && self.name() == other.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wie_util::ByteWrite;

    #[test]
    fn full_names_borrow_parts_and_reread_guest_changes() -> Result<()> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        core.map(0x1000, 0x1000)?;

        for (name, descriptor) in [("count", "I"), ("\u{ac00}", "(Ljava/lang/String;)V")] {
            let original = JavaFullName::new(name, descriptor);
            core.write_bytes(0x1000, &original.as_bytes())?;
            let parsed = JavaFullName::from_ptr(&core, 0x1000)?;
            assert_eq!((parsed.tag, parsed.name(), parsed.descriptor()), (original.tag, name, descriptor));
            assert_eq!(parsed.as_bytes(), original.as_bytes());
            assert_eq!(parsed.descriptor().as_ptr(), parsed.value.as_ptr());
            assert_eq!(parsed.name().as_ptr(), parsed.value[parsed.name_offset..].as_ptr());
            assert_eq!(format!("{parsed}"), format!("{name}{descriptor}@{}", original.tag));
            assert!(parsed == JavaFullName::new(name, descriptor));
            assert!(JavaFullName::matches(&core, 0x1000, name, descriptor)?);
            assert!(!JavaFullName::matches(&core, 0x1000, "other", descriptor)?);
            assert!(!JavaFullName::matches(&core, 0x1000, name, "J")?);
            assert_eq!(
                JavaFullName::position(&core, 0x1000, &[("other", descriptor), (name, "J"), (name, descriptor)])?,
                Some(2)
            );
            assert_eq!(JavaFullName::position(&core, 0x1000, &[("other", descriptor)])?, None);

            core.write_bytes(0x1000, &JavaFullName::new("updated", "J").as_bytes())?;
            let updated = JavaFullName::from_ptr(&core, 0x1000)?;
            assert_eq!((updated.name(), updated.descriptor()), ("updated", "J"));
            assert_eq!((parsed.name(), parsed.descriptor()), (name, descriptor));
            assert!(!JavaFullName::matches(&core, 0x1000, name, descriptor)?);
            assert!(JavaFullName::matches(&core, 0x1000, "updated", "J")?);
        }
        // Names told apart by their length, their last letter or only their text.
        core.write_bytes(0x1000, &[0; 64])?;
        core.write_bytes(0x1000, &JavaFullName::new("clip", "I").as_bytes())?;
        assert!(!JavaFullName::matches(&core, 0x1000, "clipX", "I")?);
        assert_eq!(
            JavaFullName::position(&core, 0x1000, &[("clipX", "I"), ("cli", "I"), ("clip", "I")])?,
            Some(2)
        );
        core.write_bytes(0x1000, &JavaFullName::new("clipX", "I").as_bytes())?;
        for other in ["clipY", "slipX", "clip"] {
            assert!(!JavaFullName::matches(&core, 0x1000, other, "I")?);
        }
        assert!(!JavaFullName::matches(&core, 0x1000, "clipX", "J")?);
        assert_eq!(
            JavaFullName::position(&core, 0x1000, &[("clipY", "I"), ("slipX", "I"), ("clip", "I"), ("clipX", "I")])?,
            Some(3)
        );
        let long_name = "member".repeat(12);
        for name in ["x", long_name.as_str()] {
            let bytes = JavaFullName::new(name, "I").as_bytes();
            let address = 0x10000 - bytes.len() as u32;
            core.write_bytes(address, &bytes)?;
            assert!(JavaFullName::matches(&core, address, name, "I")?);
            assert!(!JavaFullName::matches(&core, address, &format!("{name}suffix"), "I")?);
            assert_eq!(JavaFullName::position(&core, address, &[("y", "I"), (name, "I")])?, Some(1));
            assert_eq!(JavaFullName::position(&core, address, &[(&format!("{name}suffix"), "I")])?, None);
        }
        Ok(())
    }
}
