use alloc::{vec, vec::Vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.WieAudioClip
pub struct WieAudioClip;

impl WieAudioClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WieAudioClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["com/skt/m/AudioClip"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("open", "([BII)V", Self::open, Default::default()),
                JavaMethodProto::new("play", "()V", Self::play, Default::default()),
                JavaMethodProto::new("loop", "()V", Self::r#loop, Default::default()),
                JavaMethodProto::new("stop", "()V", Self::stop, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("audioHandle", "I", Default::default())],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        // 0 = not opened yet; set by open()
        jvm.put_field(&mut this, "audioHandle", "I", 0).await?;

        Ok(())
    }

    async fn open(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        buffer_size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::open({this:?}, {data:?}, {offset}, {buffer_size})");

        let raw: Vec<i8> = jvm.load_array(&data, offset as _, buffer_size as _).await?;
        let bytes: Vec<u8> = raw.into_iter().map(|b| b as u8).collect();

        match context.system().audio().load_smaf(&bytes) {
            Ok(audio_handle) => {
                jvm.put_field(&mut this, "audioHandle", "I", audio_handle as i32).await?;
            }
            Err(e) => {
                tracing::warn!("net.wie.WieAudioClip::open: unsupported audio data ({e:?})");
            }
        }

        Ok(())
    }

    async fn play(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::play({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        if audio_handle != 0 {
            let system = context.system();
            system.audio().play(system, audio_handle as u32, false).unwrap();
        }

        Ok(())
    }

    async fn r#loop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::loop({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        if audio_handle != 0 {
            let system = context.system();
            system.audio().play(system, audio_handle as u32, true).unwrap();
        }

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::stop({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        if audio_handle != 0 {
            context.system().audio().stop(audio_handle as u32);
        }

        Ok(())
    }

    async fn close(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::close({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        if audio_handle != 0 {
            context.system().audio().close(audio_handle as u32).unwrap();
        }

        Ok(())
    }
}
