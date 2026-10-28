use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Display;

// class com.ktf.kfc.GMsgBox: a message box is a form (강호동맞고2 links it beside GForm).
pub struct GMsgBox;

impl GMsgBox {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GMsgBox",
            parent_class: Some("com/ktf/kfc/GForm"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(IIII)V", Self::init_with_bounds, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Display;IIII)V",
                    Self::init_with_display,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                    Self::init_with_display_flag,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMsgBox::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "com/ktf/kfc/GForm", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_bounds(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMsgBox::<init>({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "com/ktf/kfc/GForm", "<init>", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_display(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        disp: ClassInstanceRef<Display>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMsgBox::<init>({this:?}, {disp:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(
                &this,
                "com/ktf/kfc/GForm",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIII)V",
                (disp, x, y, width, height),
            )
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_display_flag(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        disp: ClassInstanceRef<Display>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flag: bool,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMsgBox::<init>({this:?}, {disp:?}, {x}, {y}, {width}, {height}, {flag})");

        let _: () = jvm
            .invoke_special(
                &this,
                "com/ktf/kfc/GForm",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (disp, x, y, width, height, flag),
            )
            .await?;

        Ok(())
    }
}
