use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class mmpp.media.Vibration
pub struct Vibration;

impl Vibration {
    /// The API only says levels exist; five matches the handsets' 0..5 volume scale.
    const LEVELS: i32 = 5;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "mmpp/media/Vibration",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "getLevelNum",
                    "()I",
                    Self::get_level_num,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("start", "(II)V", Self::start, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn get_level_num(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("mmpp.media.Vibration::getLevelNum()");

        Ok(Self::LEVELS)
    }

    async fn start(_jvm: &Jvm, context: &mut WieJvmContext, level: i32, timeout: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.media.Vibration::start({level}, {timeout})");

        let duration_ms = timeout.max(0) as u64;
        let intensity = (level.clamp(0, Self::LEVELS) * 100 / Self::LEVELS) as u8;
        context.system().platform().vibrate(duration_ms, intensity);

        Ok(())
    }

    async fn stop(_jvm: &Jvm, context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("mmpp.media.Vibration::stop()");

        context.system().platform().vibrate(0, 0);

        Ok(())
    }
}
