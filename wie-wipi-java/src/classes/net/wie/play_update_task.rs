use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::util::{Timer, TimerTask};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::media::{Clip, PlayListener};

// class net.wie.PlayUpdateTask: a TimerTask that tells a clip's PlayListener END_OF_DATA when the
// clip's one play has run its length, unless the clip was played or stopped again since.
pub struct PlayUpdateTask;

impl PlayUpdateTask {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/PlayUpdateTask",
            parent_class: Some("java/util/TimerTask"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/media/Clip;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("run", "()V", Self::run, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("clip", "Lorg/kwis/msp/media/Clip;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("generation", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("timer", "Ljava/util/Timer;", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    pub async fn schedule(jvm: &Jvm, clip: ClassInstanceRef<Clip>, generation: i32, delay_ms: i64) -> JvmResult<()> {
        let task: ClassInstanceRef<TimerTask> = jvm
            .new_class("net/wie/PlayUpdateTask", "(Lorg/kwis/msp/media/Clip;I)V", (clip, generation))
            .await?
            .into();

        let mut timer: ClassInstanceRef<Timer> = jvm.get_static_field("net/wie/PlayUpdateTask", "timer", "Ljava/util/Timer;").await?;
        if timer.is_null() {
            timer = jvm.new_class("java/util/Timer", "()V", ()).await?.into();
            jvm.put_static_field("net/wie/PlayUpdateTask", "timer", "Ljava/util/Timer;", timer.clone())
                .await?;
        }

        jvm.invoke_virtual(&timer, "java/util/Timer", "schedule", "(Ljava/util/TimerTask;J)V", (task, delay_ms))
            .await
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        clip: ClassInstanceRef<Clip>,
        generation: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.PlayUpdateTask::<init>({this:?}, {clip:?}, {generation})");

        let _: () = jvm.invoke_special(&this, "java/util/TimerTask", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "clip", "Lorg/kwis/msp/media/Clip;", clip).await?;
        jvm.put_field(&mut this, "generation", "I", generation).await?;

        Ok(())
    }

    async fn run(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.PlayUpdateTask::run({this:?})");

        let clip: ClassInstanceRef<Clip> = jvm.get_field(&this, "clip", "Lorg/kwis/msp/media/Clip;").await?;
        let generation: i32 = jvm.get_field(&this, "generation", "I").await?;
        if Clip::generation(jvm, &clip).await? != generation {
            return Ok(());
        }

        Clip::notify(jvm, &clip, PlayListener::END_OF_DATA).await
    }
}
