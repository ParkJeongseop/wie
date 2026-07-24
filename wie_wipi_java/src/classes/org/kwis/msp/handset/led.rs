use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.LED
pub struct LED;

impl LED {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/LED",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("getCount", "()I", Self::get_count, MethodAccessFlags::STATIC),
                JavaMethodProto::new("set", "(I)V", Self::set, MethodAccessFlags::STATIC),
                JavaMethodProto::new("get", "()I", Self::get, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn get_count(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.handset.LED::getCount()");

        // no controllable LEDs on the host
        Ok(0)
    }

    async fn set(_: &Jvm, _: &mut WieJvmContext, leds: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.LED::set({leds:#x})");

        Ok(())
    }

    async fn get(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.handset.LED::get()");

        Ok(0)
    }
}
