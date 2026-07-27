use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Image;

// class com.ktf.kfc.GMenubarForm
pub struct GMenubarForm;

impl GMenubarForm {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GMenubarForm",
            parent_class: Some("com/ktf/kfc/GFormBase"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_title, Default::default()),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/Image;)V",
                    Self::init_with_title_icon,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    Self::init_with_title_path,
                    Default::default(),
                ),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMenubarForm::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "com/ktf/kfc/GFormBase", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_title(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMenubarForm::<init>({this:?}, {title:?})");

        let _: () = jvm
            .invoke_special(&this, "com/ktf/kfc/GFormBase", "<init>", "(Ljava/lang/String;)V", (title,))
            .await?;

        Ok(())
    }

    async fn init_with_title_icon(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        icon: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMenubarForm::<init>({this:?}, {title:?}, {icon:?})");

        let _: () = jvm
            .invoke_special(&this, "com/ktf/kfc/GFormBase", "<init>", "(Ljava/lang/String;)V", (title,))
            .await?;

        Ok(())
    }

    async fn init_with_title_path(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        img_path: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMenubarForm::<init>({this:?}, {title:?}, {img_path:?})");

        let _: () = jvm
            .invoke_special(&this, "com/ktf/kfc/GFormBase", "<init>", "(Ljava/lang/String;)V", (title,))
            .await?;

        Ok(())
    }
}
