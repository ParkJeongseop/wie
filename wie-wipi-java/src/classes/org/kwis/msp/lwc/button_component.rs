use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::{Object, String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    net::wie::ComponentState,
    org::kwis::msp::{lcdui::Image, lwc::ActionListener},
};

// class org.kwis.msp.lwc.ButtonComponent: a labelled (or pictured) button that reports presses to
// an ActionListener. The shell it is shown in draws it and presses it (`net.wie.ShellCard`); its
// label and listener are kept beside it (`net.wie.ComponentState`), because titles subclass it and
// their fields follow the SDK's layout; so is its picture.
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
        ComponentState::set_text(jvm, &this, label).await?;
        ComponentState::set_image(jvm, &this, image).await?;

        Ok(())
    }

    async fn set_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::setString({this:?}, {label:?})");

        ComponentState::set_text(jvm, &this, label).await
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::getString({this:?})");

        ComponentState::text(jvm, &this).await
    }

    async fn set_image(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::setImage({this:?}, {image:?})");

        ComponentState::set_image(jvm, &this, image).await
    }

    async fn get_image(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::getImage({this:?})");

        ComponentState::image(jvm, &this).await
    }

    async fn set_action_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ActionListener>,
        data: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::setActionListener({this:?}, {listener:?}, {data:?})");

        ComponentState::set_listener(jvm, &this, listener, data).await
    }
}
