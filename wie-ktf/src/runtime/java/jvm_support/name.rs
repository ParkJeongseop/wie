use alloc::{format, string::String, vec::Vec};
use core::fmt::Display;

use wie_core_arm::ArmCore;
use wie_util::{read_generic, read_null_terminated_string_bytes};

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
        let mut core = ArmCore::new(false, None)?;
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

            core.write_bytes(0x1000, &JavaFullName::new("updated", "J").as_bytes())?;
            let updated = JavaFullName::from_ptr(&core, 0x1000)?;
            assert_eq!((updated.name(), updated.descriptor()), ("updated", "J"));
            assert_eq!((parsed.name(), parsed.descriptor()), (name, descriptor));
        }
        Ok(())
    }
}
