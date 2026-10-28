use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::{Object, String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Image, lwc::ActionListener};

// class org.kwis.msp.lwc.ButtonComponent: a labelled (or pictured) button that reports presses to
// an ActionListener. Nothing draws it or presses it here, and it keeps no state of its own (titles
// subclass it; their fields follow the SDK's layout), so a title that builds one (미니고치) links and
// goes on.
pub struct ButtonComponent;

impl ButtonComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ButtonComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/Image;)V",
                    Self::init_with_label,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setImage", "(Lorg/kwis/msp/lcdui/Image;)V", Self::set_image, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getImage", "()Lorg/kwis/msp/lcdui/Image;", Self::get_image, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setActionListener",
                    "(Lorg/kwis/msp/lwc/ActionListener;Ljava/lang/Object;)V",
                    Self::set_action_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_label(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::<init>({this:?}, {label:?}, {image:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_string(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ButtonComponent::setString({this:?}, {label:?})");

        Ok(())
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("stub org.kwis.msp.lwc.ButtonComponent::getString({this:?})");

        Ok(JavaLangString::from_rust_string(jvm, "").await?.into())
    }

    async fn set_image(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ButtonComponent::setImage({this:?}, {image:?})");

        Ok(())
    }

    async fn get_image(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("stub org.kwis.msp.lwc.ButtonComponent::getImage({this:?})");

        Ok(None.into())
    }

    async fn set_action_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ActionListener>,
        data: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ButtonComponent::setActionListener({this:?}, {listener:?}, {data:?})");

        Ok(())
    }
}
