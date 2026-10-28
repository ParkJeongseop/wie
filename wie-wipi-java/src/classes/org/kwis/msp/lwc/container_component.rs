use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{
    lang::Object,
    util::{Hashtable, Vector},
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lwc::Component;

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent;

impl ContainerComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ContainerComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;)I",
                    Self::add_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("removeComponent", "(I)V", Self::remove_component_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "removeComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::remove_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("validate", "()V", Self::validate, Default::default()),
                JavaMethodProto::new(
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    Self::get_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getComponentCount", "()I", Self::get_component_count, MethodAccessFlags::PUBLIC),
            ],
            // Children per container, keyed by the container: a static table rather than an instance
            // field, because titles subclass containers and lay their own fields out after the SDK's.
            fields: vec![JavaFieldProto::new(
                "children",
                "Ljava/util/Hashtable;",
                FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
            )],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ContainerComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    /// The children added to `this`, in order, from the static table (created on first use).
    async fn children(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Vector>> {
        let mut table: ClassInstanceRef<Hashtable> = jvm
            .get_static_field("org/kwis/msp/lwc/ContainerComponent", "children", "Ljava/util/Hashtable;")
            .await?;
        if table.is_null() {
            table = jvm.new_class("java/util/Hashtable", "()V", ()).await?.into();
            jvm.put_static_field("org/kwis/msp/lwc/ContainerComponent", "children", "Ljava/util/Hashtable;", table.clone())
                .await?;
        }

        let children: ClassInstanceRef<Vector> = jvm
            .invoke_virtual(
                &table,
                "java/util/Hashtable",
                "get",
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                (this.clone(),),
            )
            .await?;
        if !children.is_null() {
            return Ok(children);
        }

        let children: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        let _: ClassInstanceRef<Object> = jvm
            .invoke_virtual(
                &table,
                "java/util/Hashtable",
                "put",
                "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                (this.clone(), children.clone()),
            )
            .await?;

        Ok(children)
    }

    /// Adds a child and answers its index. Nothing lays children out or draws them yet; the list
    /// is kept so `getComponent` answers what was added (마스터오브소드2 reads its form back).
    async fn add_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, component: ClassInstanceRef<Component>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::addComponent({this:?}, {component:?})");

        let children = Self::children(jvm, &this).await?;
        let _: () = jvm
            .invoke_virtual(&children, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (component,))
            .await?;
        let size: i32 = jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await?;

        Ok(size - 1)
    }

    async fn remove_component_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeComponent({this:?}, {index})");

        let children = Self::children(jvm, &this).await?;
        let size: i32 = jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index >= size {
            return Ok(());
        }
        jvm.invoke_virtual(&children, "java/util/Vector", "removeElementAt", "(I)V", (index,))
            .await
    }

    async fn remove_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeComponent({this:?}, {component:?})");

        let children = Self::children(jvm, &this).await?;
        let _: bool = jvm
            .invoke_virtual(&children, "java/util/Vector", "removeElement", "(Ljava/lang/Object;)Z", (component,))
            .await?;

        Ok(())
    }

    async fn get_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Component>> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::getComponent({this:?}, {index})");

        let children = Self::children(jvm, &this).await?;
        let size: i32 = jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index >= size {
            return Ok(None.into());
        }
        jvm.invoke_virtual(&children, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    async fn get_component_count(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::getComponentCount({this:?})");

        let children = Self::children(jvm, &this).await?;
        jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await
    }

    // Re-layout the container's children. LWC layout isn't wired to real
    // geometry yet, so accept it as a no-op instead of aborting the app.
    async fn validate(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.ContainerComponent::validate({this:?})");

        Ok(())
    }
}
