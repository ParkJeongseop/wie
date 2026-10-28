use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    com::ktf::kfc::{GMenubarForm, GTextListener},
    org::kwis::msp::lwc::Component,
};

// class com.ktf.kfc.GTextField: the toolkit's text box, a TextFieldComponent placed on a form.
// 강호동맞고2 builds one at start (a name editor) and could not load without the class.
pub struct GTextField;

impl GTextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GTextField",
            parent_class: Some("org/kwis/msp/lwc/TextFieldComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lcom/ktf/kfc/GMenubarForm;Ljava/lang/String;I)V",
                    Self::init_on_form,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;IIIII)V",
                    Self::init_on_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getGTextListener",
                    "()Lcom/ktf/kfc/GTextListener;",
                    Self::get_g_text_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setGTextListener",
                    "(Lcom/ktf/kfc/GTextListener;)V",
                    Self::set_g_text_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("move", "(II)V", Self::r#move, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("resize", "(II)V", Self::resize, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init_on_form(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        form: ClassInstanceRef<GMenubarForm>,
        text: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextField::<init>({this:?}, {form:?}, {text:?}, {constraint})");

        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/TextFieldComponent",
                "<init>",
                "(Ljava/lang/String;I)V",
                (text, constraint),
            )
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_on_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        parent: ClassInstanceRef<Component>,
        text: ClassInstanceRef<String>,
        constraint: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextField::<init>({this:?}, {parent:?}, {text:?}, {constraint}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/TextFieldComponent",
                "<init>",
                "(Ljava/lang/String;I)V",
                (text, constraint),
            )
            .await?;

        Ok(())
    }

    /// The listener the field notifies. Nothing notifies yet and the field keeps no state of its own
    /// (titles subclass it), so each ask gets a fresh listener for a title to configure input modes
    /// on (`setIMEModes`).
    async fn get_g_text_listener(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<GTextListener>> {
        tracing::debug!("com.ktf.kfc.GTextField::getGTextListener({this:?})");

        Ok(jvm.new_class("com/ktf/kfc/GTextListener", "()V", ()).await?.into())
    }

    async fn set_g_text_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<GTextListener>,
    ) -> JvmResult<()> {
        tracing::debug!("stub com.ktf.kfc.GTextField::setGTextListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, shown: bool) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextField::showNotify({this:?}, {shown})");

        Ok(())
    }

    async fn r#move(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextField::move({this:?}, {x}, {y})");

        Ok(())
    }

    async fn resize(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextField::resize({this:?}, {width}, {height})");

        Ok(())
    }
}
