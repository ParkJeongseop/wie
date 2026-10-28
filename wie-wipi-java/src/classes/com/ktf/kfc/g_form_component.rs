use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Display, lwc::Component};

// class com.ktf.kfc.GFormComponent: a shell that places children at given bounds
// (미니게임패밀리, 미니고치).
pub struct GFormComponent;

impl GFormComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GFormComponent",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
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
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;IIII)I",
                    Self::add_component_at,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GFormComponent::<init>({this:?})");

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
        tracing::debug!("com.ktf.kfc.GFormComponent::<init>({this:?}, {x}, {y}, {width}, {height})");

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
        tracing::debug!("com.ktf.kfc.GFormComponent::<init>({this:?}, {disp:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

    async fn add_component_at(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.ktf.kfc.GFormComponent::addComponent({this:?}, {component:?}, {x}, {y}, {width}, {height})");

        jvm.invoke_virtual(
            &this,
            "org/kwis/msp/lwc/ContainerComponent",
            "addComponent",
            "(Lorg/kwis/msp/lwc/Component;)I",
            (component,),
        )
        .await
    }
}
