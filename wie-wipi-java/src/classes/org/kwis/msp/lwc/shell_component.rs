use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{net::wie::ShellCard, org::kwis::msp::lwc::Component};

// class org.kwis.msp.lwc.ShellComponent
pub struct ShellComponent;

impl ShellComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ShellComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(IIII)V", Self::init_with_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setWorkComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::set_work_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hide", "()V", Self::hide, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ShellComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_size(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ShellComponent::<init>({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn set_work_component(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.ShellComponent::setWorkComponent({this:?}, {component:?})");

        Ok(())
    }

    /// Puts the shell on screen on a card of its own (`net.wie.ShellCard`), which draws it through
    /// its own `paint` and hands it the keys. 학교가는길's whole game is a shell.
    async fn show(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::show({this:?})");

        ShellCard::show(jvm, &this).await
    }

    async fn hide(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::hide({this:?})");

        ShellCard::hide(jvm, &this).await
    }

    /// Lays the children out again after they changed; nothing is laid out here (미니고치 calls it
    /// on its form at start).
    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::layout({this:?})");

        Ok(())
    }

    /// Whether the shell is on screen (미니고치 asks before showing its form).
    async fn is_shown(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::isShown({this:?})");

        Ok(!ShellCard::card_of(jvm, &this).await?.is_null())
    }

    async fn service_repaints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::serviceRepaints({this:?})");

        let card = ShellCard::card_of(jvm, &this).await?;
        if card.is_null() {
            return Ok(());
        }

        jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "serviceRepaints", "()V", ()).await
    }
}
