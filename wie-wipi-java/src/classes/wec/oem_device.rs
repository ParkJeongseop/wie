use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::MethodAccessFlags;
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class wec.OEMDevice
pub struct OEMDevice;

impl OEMDevice {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/OEMDevice",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("getAddressBook", "()Lwec/AddressBook;", Self::get_address_book, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getSYSTheme", "()Lwec/SYSTheme;", Self::get_sys_theme, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn get_address_book(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Object>> {
        tracing::warn!("stub wec.OEMDevice::getAddressBook()");

        // address book is not supported on the host; the API allows null
        Ok(None.into())
    }

    async fn get_sys_theme(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Object>> {
        tracing::warn!("stub wec.OEMDevice::getSYSTheme()");

        // theme is not supported on the host; the API allows null
        Ok(None.into())
    }
}
