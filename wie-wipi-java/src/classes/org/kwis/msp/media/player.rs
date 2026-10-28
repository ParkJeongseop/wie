use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::util::Vector;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use wie_backend::Event;
use wie_jvm_support::JvmSupport;

use crate::classes::org::kwis::msp::media::{BaseClip, Clip, PlayListener};

// class org.kwis.msp.media.Player
pub struct Player;

impl Player {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Player",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "pause",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::pause,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::stop,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "resume",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::resume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                    Self::play,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "record",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::record,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/Clip;Z)Z",
                    Self::play_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/Clip;)Z",
                    Self::stop_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/Clip;Z)Z",
                    Self::play_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/Clip;)Z",
                    Self::stop_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "resume",
                    "(Lorg/kwis/msp/media/Clip;)Z",
                    Self::resume_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            // Clips whose end-of-data is still to be delivered: the event that delivers it holds
            // the clip past any Java reference, so a static root keeps the collector off it.
            fields: vec![JavaFieldProto::new(
                "pendingClips",
                "Ljava/util/Vector;",
                FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
            )],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn pause(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::pause({clip:?})");

        Ok(false)
    }

    async fn stop(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({clip:?})");

        Ok(false)
    }

    async fn resume(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::resume({clip:?})");

        Ok(false)
    }

    async fn play(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>, repeat: bool) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        Ok(false)
    }

    async fn record(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::record({clip:?})");

        Ok(false)
    }

    async fn play_clip(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<Clip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        if clip.is_null() {
            return Ok(false);
        }
        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "net/wie/SmafPlayer", "start", "(Z)V", (repeat,)).await?;

            // A clip played once ends when its sequence does; the sink does not report it, so the
            // end is scheduled from the duration (microseconds, -1 when unknown) on the system event
            // queue, like an Alert's timeout. Only the end is delivered, and not from inside play or
            // stop (a title's handler commonly stops or replays the clip) nor from a Java thread of
            // its own (a second thread races a title's own loading thread: 트랜스포머's second launch
            // threw in its loader once a timer thread ran beside it).
            let generation = Clip::next_generation(jvm, &clip).await?;
            if !repeat {
                let duration: i64 = jvm
                    .invoke_virtual(&player, "javax/microedition/media/Player", "getDuration", "()J", ())
                    .await?;
                if duration > 0 {
                    let pending = Self::pending_clips(jvm).await?;
                    let _: () = jvm
                        .invoke_virtual(&pending, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (clip.clone(),))
                        .await?;

                    let due = context.system().platform().now() + (duration / 1000) as u64;
                    let timer_jvm = jvm.clone();
                    context.system().event_queue().push(Event::timer(due, move || {
                        let jvm = timer_jvm;
                        async move {
                            let result: JvmResult<()> = async {
                                let pending = Self::pending_clips(&jvm).await?;
                                let _: bool = jvm
                                    .invoke_virtual(&pending, "java/util/Vector", "removeElement", "(Ljava/lang/Object;)Z", (clip.clone(),))
                                    .await?;
                                if Clip::generation(&jvm, &clip).await? != generation {
                                    return Ok(());
                                }
                                Clip::notify(&jvm, &clip, PlayListener::END_OF_DATA).await
                            }
                            .await;

                            match result {
                                Ok(()) => Ok(()),
                                Err(error) => Err(JvmSupport::to_wie_err(&jvm, error).await),
                            }
                        }
                    }));
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn pending_clips(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Vector>> {
        let pending: ClassInstanceRef<Vector> = jvm
            .get_static_field("org/kwis/msp/media/Player", "pendingClips", "Ljava/util/Vector;")
            .await?;
        if !pending.is_null() {
            return Ok(pending);
        }
        let pending: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        jvm.put_static_field("org/kwis/msp/media/Player", "pendingClips", "Ljava/util/Vector;", pending.clone())
            .await?;

        Ok(pending)
    }

    async fn stop_clip(jvm: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::stop({clip:?})");

        if clip.is_null() {
            return Ok(false);
        }
        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "stop", "()V", ()).await?;

            Clip::next_generation(jvm, &clip).await?;

            return Ok(true);
        }

        Ok(false)
    }

    async fn resume_clip(jvm: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::resume({clip:?})");

        if clip.is_null() {
            return Ok(false);
        }
        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            // start(Z) is SmafPlayer's own entry point (see play_clip); the MIDP Player
            // interface only declares start()V, so looking it up there fails.
            let _: () = jvm.invoke_virtual(&player, "net/wie/SmafPlayer", "start", "(Z)V", (false,)).await?;

            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::media::{BaseClip, Clip},
        get_protos,
    };

    #[test]
    fn test_base_clip_overloads_return_false() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let clip: ClassInstanceRef<BaseClip> = jvm.new_class("org/kwis/msp/media/BaseClip", "()V", ()).await?.into();

            let paused: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "pause", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let resumed: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let played: bool = jvm
                .invoke_static(
                    "org/kwis/msp/media/Player",
                    "play",
                    "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                    (clip.clone(), true),
                )
                .await?;
            let recorded: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "record", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip,))
                .await?;

            assert!(!paused);
            assert!(!stopped);
            assert!(!resumed);
            assert!(!played);
            assert!(!recorded);

            Ok(())
        })
    }

    #[test]
    fn test_clip_playback_uses_repeat_overload() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
            let data = jvm.instantiate_array("B", 0).await?;
            let clip: ClassInstanceRef<Clip> = jvm
                .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data))
                .await?
                .into();

            let played: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (clip.clone(), true))
                .await?;
            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip,))
                .await?;

            assert!(played);
            assert!(stopped);

            Ok(())
        })
    }
}
