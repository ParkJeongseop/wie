use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class mmpp.media.BackLight
//
// The host's display has no backlight to switch: the calls are accepted and change nothing.
pub struct BackLight;

impl BackLight {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "mmpp/media/BackLight",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("on", "(I)V", Self::on, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("off", "()V", Self::off, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "numColors",
                    "()I",
                    Self::num_colors,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("getColor", "()I", Self::get_color, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn on(_jvm: &Jvm, _context: &mut WieJvmContext, timeout: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.media.BackLight::on({timeout})");

        Ok(())
    }

    async fn off(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("mmpp.media.BackLight::off()");

        Ok(())
    }

    async fn num_colors(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("mmpp.media.BackLight::numColors()");

        Ok(1)
    }

    async fn set_color(_jvm: &Jvm, _context: &mut WieJvmContext, rgb: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.media.BackLight::setColor({rgb:#x})");

        Ok(())
    }

    async fn get_color(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("mmpp.media.BackLight::getColor()");

        Ok(0xffffff)
    }
}
