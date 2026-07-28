use alloc::{string::String, vec::Vec};
use core::fmt::Display;

use wie_core_arm::ArmCore;
use wie_util::{read_generic, read_null_terminated_string_bytes};

use super::Result;

#[derive(Clone)]
pub struct JavaFullName {
    pub tag: u8,
    pub name: String,
    pub descriptor: String,
}

impl JavaFullName {
    /// KTF의 메서드/필드 이름 구조는 1바이트 tag를 갖는다: "descriptor+name" 문자열의
    /// Java 문자열 해시(h*31+c) 하위 바이트. client.bin이 테이블 스캔 시 이 tag로
    /// 메서드를 찾으므로 0으로 쓰면 이름 조회 없는 디스패치가 엉뚱한 오버로드에
    /// 도달한다 (놈3: <init>(Ljava/lang/String;)V 대신 <init>([CII)V로 점프).
    pub fn new(name: String, descriptor: String) -> Self {
        let tag = descriptor
            .bytes()
            .chain([b'+'])
            .chain(name.bytes())
            .fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32)) as u8;

        Self { tag, name, descriptor }
    }

    pub fn from_ptr(core: &ArmCore, ptr: u32) -> Result<Self> {
        let tag = read_generic(core, ptr)?;

        let value = read_null_terminated_string_bytes(core, ptr + 1)?;
        let value = String::from_utf8(value).unwrap();
        let mut values = value.split('+');

        let descriptor = values.next().unwrap().into();
        let name = values.next().unwrap().into();

        Ok(JavaFullName { tag, name, descriptor })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.tag);
        bytes.extend_from_slice(self.descriptor.as_bytes());
        bytes.push(b'+');
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);

        bytes
    }
}

impl Display for JavaFullName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name.fmt(f)?;
        self.descriptor.fmt(f)?;
        write!(f, "@{}", self.tag)?;

        Ok(())
    }
}

impl PartialEq for JavaFullName {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor && self.name == other.name
    }
}
