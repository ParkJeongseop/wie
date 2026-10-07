use alloc::{format, vec};

use jvm::{
    Array, ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class mmpp.media.MediaPlayer
//
// Plays one SMAF (.mmf) clip at a time, taken from a jar resource or a byte array. `audioHandle` is
// the clip loaded into the host's audio system, -1 while there is none.
pub struct MediaPlayer;

impl MediaPlayer {
    /// Volume levels are the strings "0" (silent) to "5".
    const DEFAULT_VOLUME: &'static str = "3";
    const SILENT: &'static str = "0";

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "mmpp/media/MediaPlayer",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setMediaLocation",
                    "(Ljava/lang/String;)V",
                    Self::set_media_location,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setMediaSource", "([B)V", Self::set_media_source, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMediaSource", "([BII)V", Self::set_media_source_range, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isPlayBackLoop", "()Z", Self::is_play_back_loop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPlayBackLoop", "(Z)V", Self::set_play_back_loop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getVolumeLevel",
                    "()Ljava/lang/String;",
                    Self::get_volume_level,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setVolumeLevel",
                    "(Ljava/lang/String;)V",
                    Self::set_volume_level,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("start", "()V", Self::start, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("pause", "()V", Self::pause, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("resume", "()V", Self::resume, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("audioHandle", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("playBackLoop", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("volumeLevel", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let volume = JavaLangString::from_rust_string(jvm, Self::DEFAULT_VOLUME).await?;
        jvm.put_field(&mut this, "audioHandle", "I", -1).await?;
        jvm.put_field(&mut this, "volumeLevel", "Ljava/lang/String;", volume).await
    }

    async fn set_media_location(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        location: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::setMediaLocation({this:?}, {location:?})");

        let location = JavaLangString::to_rust_string(jvm, &location).await?;
        let class_loader = JavaLangClassLoader::get_system_class_loader(jvm).await?;
        let Some(stream) = JavaLangClassLoader::get_resource_as_stream(jvm, &class_loader, &location).await? else {
            return Err(jvm.exception("java/io/IOException", &format!("Media not found: {location}")).await);
        };
        let data = JavaIoInputStream::read_until_end(jvm, &stream).await?;

        Self::load(jvm, context, this, &data).await
    }

    async fn set_media_source(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::setMediaSource({this:?}, {buffer:?})");

        let length = jvm.array_length(&buffer).await? as i32;

        Self::set_media_source_range(jvm, context, this, buffer, 0, length).await
    }

    async fn set_media_source_range(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::setMediaSource({this:?}, {buffer:?}, {offset}, {length})");

        let mut data = vec![0; length.max(0) as usize];
        jvm.array_raw_buffer(&buffer).await?.read(offset.max(0) as _, &mut data)?;

        Self::load(jvm, context, this, &data).await
    }

    /// Replaces the loaded clip. Data the audio system cannot decode leaves the player without a clip.
    async fn load(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, data: &[u8]) -> JvmResult<()> {
        let previous: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        let system = context.system();
        if previous >= 0 {
            system.audio().stop(previous as _);
            let _ = system.audio().close(previous as _);
        }

        let audio_handle = match system.audio().load_smaf(data) {
            Ok(audio_handle) => audio_handle as i32,
            Err(error) => {
                tracing::warn!("mmpp.media.MediaPlayer: cannot load {} bytes of media: {error:?}", data.len());
                -1
            }
        };

        jvm.put_field(&mut this, "audioHandle", "I", audio_handle).await
    }

    async fn is_play_back_loop(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("mmpp.media.MediaPlayer::isPlayBackLoop({this:?})");

        jvm.get_field(&this, "playBackLoop", "Z").await
    }

    async fn set_play_back_loop(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: bool) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::setPlayBackLoop({this:?}, {value})");

        jvm.put_field(&mut this, "playBackLoop", "Z", value).await
    }

    async fn get_volume_level(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("mmpp.media.MediaPlayer::getVolumeLevel({this:?})");

        jvm.get_field(&this, "volumeLevel", "Ljava/lang/String;").await
    }

    async fn set_volume_level(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        volume: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::setVolumeLevel({this:?}, {volume:?})");

        if volume.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "volume is null").await);
        }
        jvm.put_field(&mut this, "volumeLevel", "Ljava/lang/String;", volume).await?;

        // the host mixes every clip at one level, so the only level it can honour is silence
        if Self::is_silent(jvm, &this).await? {
            Self::stop(jvm, context, this).await?;
        }

        Ok(())
    }

    async fn is_silent(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<bool> {
        let volume: ClassInstanceRef<String> = jvm.get_field(this, "volumeLevel", "Ljava/lang/String;").await?;

        Ok(JavaLangString::to_rust_string(jvm, &volume).await?.trim() == Self::SILENT)
    }

    async fn start(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::start({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        if audio_handle < 0 || Self::is_silent(jvm, &this).await? {
            return Ok(());
        }
        let repeat: bool = jvm.get_field(&this, "playBackLoop", "Z").await?;

        if let Err(error) = context.system().audio().play(audio_handle as _, repeat) {
            tracing::warn!("mmpp.media.MediaPlayer: cannot play: {error:?}");
        }

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::stop({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        if audio_handle >= 0 {
            context.system().audio().stop(audio_handle as _);
        }

        Ok(())
    }

    // The audio system has no pause position: pausing stops the clip and resuming restarts it.
    async fn pause(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::pause({this:?})");

        Self::stop(jvm, context, this).await
    }

    async fn resume(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.media.MediaPlayer::resume({this:?})");

        Self::start(jvm, context, this).await
    }
}
