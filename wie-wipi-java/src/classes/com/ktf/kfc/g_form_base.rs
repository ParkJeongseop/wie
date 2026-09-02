use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.GFormBase
pub struct GFormBase;

impl GFormBase {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GFormBase",
            parent_class: Some("com/ktf/kfc/GForm"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_title, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GFormBase::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "com/ktf/kfc/GForm", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_title(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GFormBase::<init>({this:?}, {title:?})");

        let _: () = jvm
            .invoke_special(&this, "com/ktf/kfc/GForm", "<init>", "(Ljava/lang/String;)V", (title,))
            .await?;

        Ok(())
    }
}
