use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{
    lang::{Object, String},
    util::Hashtable,
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Image, lwc::ActionListener};

// class net.wie.ComponentState: what an lwc component was given to show (a label's or a button's
// text, a button's picture, a shell's title), to call (a button's listener) and where it was placed.
// It is kept beside the component in a static table rather than on it, because titles subclass
// components and lay their own fields out after the SDK's.
pub struct ComponentState;

impl ComponentState {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/ComponentState",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC)],
            fields: vec![
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("image", "Lorg/kwis/msp/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("bounds", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("listener", "Lorg/kwis/msp/lwc/ActionListener;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("data", "Ljava/lang/Object;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("states", "Ljava/util/Hashtable;", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.ComponentState::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn states(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Hashtable>> {
        let mut table: ClassInstanceRef<Hashtable> = jvm.get_static_field("net/wie/ComponentState", "states", "Ljava/util/Hashtable;").await?;
        if table.is_null() {
            table = jvm.new_class("java/util/Hashtable", "()V", ()).await?.into();
            jvm.put_static_field("net/wie/ComponentState", "states", "Ljava/util/Hashtable;", table.clone())
                .await?;
        }

        Ok(table)
    }

    /// The state kept for `component`; null when nothing was kept for it.
    async fn find<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>) -> JvmResult<ClassInstanceRef<Self>> {
        let table = Self::states(jvm).await?;

        jvm.invoke_virtual(
            &table,
            "java/util/Hashtable",
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            (component.clone(),),
        )
        .await
    }

    /// The state kept for `component`, started empty on first use.
    async fn of<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>) -> JvmResult<ClassInstanceRef<Self>> {
        let state = Self::find(jvm, component).await?;
        if !state.is_null() {
            return Ok(state);
        }

        let state: ClassInstanceRef<Self> = jvm.new_class("net/wie/ComponentState", "()V", ()).await?.into();
        let table = Self::states(jvm).await?;
        let _: ClassInstanceRef<Object> = jvm
            .invoke_virtual(
                &table,
                "java/util/Hashtable",
                "put",
                "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                (component.clone(), state.clone()),
            )
            .await?;

        Ok(state)
    }

    /// The text `component` shows; null when it was given none.
    pub async fn text<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>) -> JvmResult<ClassInstanceRef<String>> {
        let state = Self::find(jvm, component).await?;
        if state.is_null() {
            return Ok(None.into());
        }

        jvm.get_field(&state, "net/wie/ComponentState", "text", "Ljava/lang/String;").await
    }

    pub async fn set_text<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        let mut state = Self::of(jvm, component).await?;

        jvm.put_field(&mut state, "net/wie/ComponentState", "text", "Ljava/lang/String;", text)
            .await
    }

    /// The picture `component` shows; null when it was given none.
    pub async fn image<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>) -> JvmResult<ClassInstanceRef<Image>> {
        let state = Self::find(jvm, component).await?;
        if state.is_null() {
            return Ok(None.into());
        }

        jvm.get_field(&state, "net/wie/ComponentState", "image", "Lorg/kwis/msp/lcdui/Image;")
            .await
    }

    pub async fn set_image<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        let mut state = Self::of(jvm, component).await?;

        jvm.put_field(&mut state, "net/wie/ComponentState", "image", "Lorg/kwis/msp/lcdui/Image;", image)
            .await
    }

    /// Where `component` was placed in its shell, as x, y, width and height; `None` when the title
    /// left that to the layout.
    pub async fn bounds<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>) -> JvmResult<Option<[i32; 4]>> {
        let state = Self::find(jvm, component).await?;
        if state.is_null() {
            return Ok(None);
        }
        let bounds: ClassInstanceRef<Array<i32>> = jvm.get_field(&state, "net/wie/ComponentState", "bounds", "[I").await?;
        if bounds.is_null() {
            return Ok(None);
        }
        let bounds: Vec<i32> = jvm.load_array(&bounds, 0, 4).await?;

        Ok(Some([bounds[0], bounds[1], bounds[2], bounds[3]]))
    }

    pub async fn set_bounds<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>, bounds: [i32; 4]) -> JvmResult<()> {
        let mut state = Self::of(jvm, component).await?;
        let mut array = jvm.instantiate_array("I", bounds.len()).await?;
        jvm.store_array(&mut array, 0, bounds).await?;

        jvm.put_field(&mut state, "net/wie/ComponentState", "bounds", "[I", array).await
    }

    /// The listener `component` calls and the object it passes; both null when it was given none.
    pub async fn listener<T: Send + Sync>(
        jvm: &Jvm,
        component: &ClassInstanceRef<T>,
    ) -> JvmResult<(ClassInstanceRef<ActionListener>, ClassInstanceRef<Object>)> {
        let state = Self::find(jvm, component).await?;
        if state.is_null() {
            return Ok((None.into(), None.into()));
        }

        let listener = jvm
            .get_field(&state, "net/wie/ComponentState", "listener", "Lorg/kwis/msp/lwc/ActionListener;")
            .await?;
        let data = jvm.get_field(&state, "net/wie/ComponentState", "data", "Ljava/lang/Object;").await?;

        Ok((listener, data))
    }

    pub async fn set_listener<T: Send + Sync>(
        jvm: &Jvm,
        component: &ClassInstanceRef<T>,
        listener: ClassInstanceRef<ActionListener>,
        data: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        let mut state = Self::of(jvm, component).await?;

        jvm.put_field(
            &mut state,
            "net/wie/ComponentState",
            "listener",
            "Lorg/kwis/msp/lwc/ActionListener;",
            listener,
        )
        .await?;
        jvm.put_field(&mut state, "net/wie/ComponentState", "data", "Ljava/lang/Object;", data)
            .await
    }
}
