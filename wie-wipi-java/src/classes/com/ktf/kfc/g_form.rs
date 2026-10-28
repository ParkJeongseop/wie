use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Display;

// class com.ktf.kfc.GForm: the vendor widget toolkit's window, a shell with children. Nothing
// draws a widget UI here; the methods keep their contract so a title that opens a form (a name
// editor, a message box) continues. `doModal` shows the form and waits for it to be dismissed:
// nothing presents it and nobody dismisses it, so it closes at once and answers zero, what a
// dialog closed without a choice answers.
pub struct GForm;

impl GForm {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GForm",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_title, Default::default()),
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
                JavaMethodProto::new("doModal", "()I", Self::do_modal, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hide", "()V", Self::hide, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("move", "(II)V", Self::r#move, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("resize", "(II)V", Self::resize, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTransparent", "(Z)V", Self::set_transparent, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPluginLayer", "(II)V", Self::set_plugin_layer, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPluginPosition", "(I)I", Self::get_plugin_position, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_title(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::<init>({this:?}, {title:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

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
        tracing::debug!("com.ktf.kfc.GForm::<init>({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

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
        tracing::debug!("com.ktf.kfc.GForm::<init>({this:?}, {disp:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

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
        tracing::debug!("com.ktf.kfc.GForm::<init>({this:?}, {disp:?}, {x}, {y}, {width}, {height}, {flag})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

    async fn do_modal(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.ktf.kfc.GForm::doModal({this:?}): nothing presents the form, closing it at once");

        Ok(0)
    }

    async fn show(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub com.ktf.kfc.GForm::show({this:?})");

        Ok(())
    }

    async fn hide(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub com.ktf.kfc.GForm::hide({this:?})");

        Ok(())
    }

    /// Nothing presents a form, so none is shown. (Titles subclass GForm, so it keeps no state of
    /// its own: their fields follow the SDK's layout.)
    async fn is_shown(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("com.ktf.kfc.GForm::isShown({this:?})");

        Ok(false)
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, shown: bool) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::showNotify({this:?}, {shown})");

        Ok(())
    }

    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::layout({this:?})");

        Ok(())
    }

    async fn r#move(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::move({this:?}, {x}, {y})");

        Ok(())
    }

    async fn resize(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::resize({this:?}, {width}, {height})");

        Ok(())
    }

    async fn set_transparent(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, transparent: bool) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::setTransparent({this:?}, {transparent})");

        Ok(())
    }

    async fn set_plugin_layer(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, layer: i32, position: i32) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::setPluginLayer({this:?}, {layer}, {position})");

        Ok(())
    }

    async fn get_plugin_position(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, layer: i32) -> JvmResult<i32> {
        tracing::debug!("com.ktf.kfc.GForm::getPluginPosition({this:?}, {layer})");

        Ok(0)
    }
}
