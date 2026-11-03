use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{net::wie::ShellCard, org::kwis::msp::lwc::EventListener};

// class org.kwis.msp.lwc.Component
pub struct Component;

impl Component {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/Component",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("focusNotify", "(Z)V", Self::focus_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("configure", "(IIIII)V", Self::configure, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFocus", "()V", Self::set_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasFocus", "()Z", Self::has_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getX", "()I", Self::get_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getY", "()I", Self::get_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setEventListener",
                    "(Lorg/kwis/msp/lwc/EventListener;Ljava/lang/Object;)V",
                    Self::set_event_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setBackground", "(I)V", Self::set_background, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setForeground", "(I)V", Self::set_foreground, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("focused", "Z", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.Component::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::keyNotify({this:?}, {type:?}, {chr:?})");

        Ok(true)
    }

    async fn focus_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::focusNotify({this:?}, {focus:?})");

        jvm.put_field(&mut this, "focused", "Z", focus).await?;

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::showNotify({this:?}, {show:?})");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn configure(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32, mask: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::configure({this:?}, {x}, {y}, {w}, {h}, {mask})");

        Ok(())
    }

    async fn set_focus(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setFocus({this:?})");

        jvm.put_field(&mut this, "focused", "Z", true).await?;

        Ok(())
    }

    async fn has_focus(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::hasFocus({this:?})");

        jvm.get_field(&this, "focused", "Z").await
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({this:?})");

        Ok(0)
    }

    async fn get_width(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getWidth({this:?})");

        Ok(0)
    }

    async fn get_x(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getX({this:?})");

        Ok(0)
    }

    async fn get_y(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getY({this:?})");

        Ok(0)
    }

    /// Components send no events yet, so there is nothing to keep the listener for.
    async fn set_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<EventListener>,
        param: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setEventListener({this:?}, {listener:?}, {param:?})");

        Ok(())
    }

    /// Components are not painted yet, so their background color has no effect.
    async fn set_background(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setBackground({this:?}, {color:#x})");

        Ok(())
    }

    /// Components are not painted yet, so their foreground color has no effect.
    async fn set_foreground(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setForeground({this:?}, {color:#x})");

        Ok(())
    }

    // LWC widget painting isn't wired to the framebuffer yet, so repaint has
    // nothing to flush — but leaving it unimplemented aborted whole apps with
    // "method not found". Accept it as a no-op so the app keeps running (and
    // draws through whatever other path it uses).
    /// Repaints the card of a shown shell; a component inside one is not tracked to its shell yet.
    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::repaint({this:?})");

        let card = ShellCard::card_of(jvm, &this).await?;
        if card.is_null() {
            return Ok(());
        }

        jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "()V", ()).await
    }

    async fn repaint_with_area(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::repaint({this:?}, {x}, {y}, {w}, {h})");

        let card = ShellCard::card_of(jvm, &this).await?;
        if card.is_null() {
            return Ok(());
        }

        jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (x, y, w, h))
            .await
    }
}
