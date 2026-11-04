use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class m.V3
pub struct V3;

impl V3 {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "m/V3",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(III)V", Self::init_with_coordinates, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("set", "(III)V", Self::set, MethodAccessFlags::PUBLIC),
            ],
            // Titles read and write the coordinates directly.
            fields: vec![
                JavaFieldProto::new("x", "I", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("y", "I", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("z", "I", FieldAccessFlags::PUBLIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.V3::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn init_with_coordinates(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::debug!("m.V3::<init>({this:?}, {x}, {y}, {z})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        Self::write(jvm, &this, [x, y, z]).await
    }

    async fn set(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        tracing::debug!("m.V3::set({this:?}, {x}, {y}, {z})");

        Self::write(jvm, &this, [x, y, z]).await
    }

    pub async fn read(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<[i32; 3]> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "vector is null").await);
        }
        Ok([
            jvm.get_field(this, "m/V3", "x", "I").await?,
            jvm.get_field(this, "m/V3", "y", "I").await?,
            jvm.get_field(this, "m/V3", "z", "I").await?,
        ])
    }

    pub async fn write(jvm: &Jvm, this: &ClassInstanceRef<Self>, [x, y, z]: [i32; 3]) -> JvmResult<()> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "vector is null").await);
        }
        let mut this = this.clone();
        jvm.put_field(&mut this, "m/V3", "x", "I", x).await?;
        jvm.put_field(&mut this, "m/V3", "y", "I", y).await?;
        jvm.put_field(&mut this, "m/V3", "z", "I", z).await
    }
}
