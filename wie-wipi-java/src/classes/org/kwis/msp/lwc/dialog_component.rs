use alloc::{string::String as RustString, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Graphics, lwc::Component};

// Values from the WIPI Java API field list. javac inlines them into callers, so
// they are the contract even though titles never read the fields themselves.
const TYPE_NONE: i32 = 0;
const TYPE_OK: i32 = 1;
const TYPE_OK_CANCEL: i32 = 2;
const DLG_TIMEOUT: i32 = 10;
const DLG_OK: i32 = 11;
const DLG_CANCEL: i32 = 12;
const OK_BUTTON: i32 = 20;
const CANCEL_BUTTON: i32 = 21;
const TIMEOUT_INFINITE: i32 = -1;
// "기본적으로 화면에 출력되는 시간은 3초" — the documented display time of a
// dialog without buttons.
const DEFAULT_TIMEOUT_MILLIS: i32 = 3000;

// class org.kwis.msp.lwc.DialogComponent
pub struct DialogComponent;

impl DialogComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/DialogComponent",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
                    Self::init_with_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;IIIII)V",
                    Self::init_with_bounds,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("doModal", "()I", Self::do_modal, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getActionState", "()I", Self::get_action_state, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTimeout", "()I", Self::get_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTimeout", "(I)V", Self::set_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setType", "(I)V", Self::set_type, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setButtonString",
                    "(ILjava/lang/String;)V",
                    Self::set_button_string,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paintFrame",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    Self::paint_frame,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("TYPE_NONE", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("TYPE_OK", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("TYPE_OK_CANCEL", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("DLG_TIMEOUT", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("DLG_OK", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("DLG_CANCEL", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("OK_BUTTON", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("CANCEL_BUTTON", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("TIMEOUT_INFINITE", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("actionState", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("dialogType", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("timeout", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("title", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("dataComponent", "Lorg/kwis/msp/lwc/Component;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("okString", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cancelString", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<clinit>");

        let class = "org/kwis/msp/lwc/DialogComponent";
        jvm.put_static_field(class, "TYPE_NONE", "I", TYPE_NONE).await?;
        jvm.put_static_field(class, "TYPE_OK", "I", TYPE_OK).await?;
        jvm.put_static_field(class, "TYPE_OK_CANCEL", "I", TYPE_OK_CANCEL).await?;
        jvm.put_static_field(class, "DLG_TIMEOUT", "I", DLG_TIMEOUT).await?;
        jvm.put_static_field(class, "DLG_OK", "I", DLG_OK).await?;
        jvm.put_static_field(class, "DLG_CANCEL", "I", DLG_CANCEL).await?;
        jvm.put_static_field(class, "OK_BUTTON", "I", OK_BUTTON).await?;
        jvm.put_static_field(class, "CANCEL_BUTTON", "I", CANCEL_BUTTON).await?;
        jvm.put_static_field(class, "TIMEOUT_INFINITE", "I", TIMEOUT_INFINITE).await?;

        Ok(())
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, r#type: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {type})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;
        Self::store(jvm, &mut this, None.into(), None.into(), r#type).await
    }

    async fn init_with_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        r#type: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {component:?}, {title:?}, {type})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;
        Self::store(jvm, &mut this, component, title, r#type).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_bounds(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        r#type: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {component:?}, {title:?}, {type}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", (x, y, width, height))
            .await?;
        Self::store(jvm, &mut this, component, title, r#type).await
    }

    async fn store(
        jvm: &Jvm,
        this: &mut ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        r#type: i32,
    ) -> JvmResult<()> {
        jvm.put_field(this, "dataComponent", "Lorg/kwis/msp/lwc/Component;", component).await?;
        jvm.put_field(this, "title", "Ljava/lang/String;", title).await?;
        Self::apply_type(jvm, this, r#type).await
    }

    // A dialog without buttons closes by itself after the documented three
    // seconds; one with buttons waits for them.
    async fn apply_type(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, r#type: i32) -> JvmResult<()> {
        let timeout = if r#type == TYPE_NONE { DEFAULT_TIMEOUT_MILLIS } else { TIMEOUT_INFINITE };
        jvm.put_field(this, "dialogType", "I", r#type).await?;
        jvm.put_field(this, "timeout", "I", timeout).await?;

        Ok(())
    }

    async fn title(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<RustString> {
        let title: ClassInstanceRef<String> = jvm.get_field(this, "title", "Ljava/lang/String;").await?;
        if title.is_null() {
            return Ok(RustString::new());
        }

        JavaLangString::to_rust_string(jvm, &title).await
    }

    // TODO: the modal answer. See the comment at the call site in do_modal.
    async fn do_modal(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let r#type: i32 = jvm.get_field(&this, "dialogType", "I").await?;
        let title = Self::title(jvm, &this).await?;
        let action = match r#type {
            TYPE_NONE => DLG_TIMEOUT,
            _ => DLG_OK,
        };
        tracing::warn!("stub org.kwis.msp.lwc.DialogComponent::doModal({this:?}) type {type} title {title:?} -> {action}");

        jvm.put_field(&mut this, "actionState", "I", action).await?;

        Ok(action)
    }

    async fn show(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let title = Self::title(jvm, &this).await?;
        tracing::warn!("stub org.kwis.msp.lwc.DialogComponent::show({this:?}) title {title:?}");

        Ok(())
    }

    async fn get_action_state(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::getActionState({this:?})");

        jvm.get_field(&this, "actionState", "I").await
    }

    async fn get_timeout(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::getTimeout({this:?})");

        jvm.get_field(&this, "timeout", "I").await
    }

    async fn set_timeout(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, timeout: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::setTimeout({this:?}, {timeout})");

        jvm.put_field(&mut this, "timeout", "I", timeout).await
    }

    async fn set_type(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, r#type: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::setType({this:?}, {type})");

        Self::apply_type(jvm, &mut this, r#type).await
    }

    async fn set_button_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        button_type: i32,
        label: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::setButtonString({this:?}, {button_type}, {label:?})");

        match button_type {
            OK_BUTTON => jvm.put_field(&mut this, "okString", "Ljava/lang/String;", label).await,
            CANCEL_BUTTON => jvm.put_field(&mut this, "cancelString", "Ljava/lang/String;", label).await,
            _ => {
                tracing::warn!("org.kwis.msp.lwc.DialogComponent::setButtonString: unknown button type {button_type}");
                Ok(())
            }
        }
    }

    // Child layout; LWC geometry is not modeled yet (see ContainerComponent::validate).
    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.DialogComponent::layout({this:?})");

        Ok(())
    }

    async fn paint_frame(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.DialogComponent::paintFrame({this:?}, {graphics:?})");

        Ok(())
    }
}
