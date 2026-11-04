use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::{ONE, V3};

/// The twelve cells in row-major order: three rows of a fixed-point rotation, each followed by a
/// translation in a coordinate's own units.
const CELLS: [&str; 12] = ["m00", "m01", "m02", "m03", "m10", "m11", "m12", "m13", "m20", "m21", "m22", "m23"];

pub const IDENTITY: [i32; 12] = [ONE as i32, 0, 0, 0, 0, ONE as i32, 0, 0, 0, 0, ONE as i32, 0];

// class m.A3
pub struct A3;

impl A3 {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "m/A3",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("ident", "()V", Self::ident, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("set", "(Lm/A3;)V", Self::set, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("mul", "(Lm/A3;Lm/A3;)V", Self::mul, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("trans", "(Lm/V3;Lm/V3;)V", Self::trans, MethodAccessFlags::PUBLIC),
            ],
            fields: CELLS
                .iter()
                .map(|name| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC))
                .collect(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // A new transform is the identity, which a camera built from a rotation and a translation
    // starts from.
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.A3::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        Self::write(jvm, &this, IDENTITY).await
    }

    async fn ident(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.A3::ident({this:?})");

        Self::write(jvm, &this, IDENTITY).await
    }

    async fn set(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, source: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.A3::set({this:?}, {source:?})");

        let cells = Self::read(jvm, &source).await?;
        Self::write(jvm, &this, cells).await
    }

    /// `this = left * right`: the rotation is renormalized by the fixed point and `left`'s
    /// translation is carried through `right`'s rotation.
    async fn mul(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        left: ClassInstanceRef<Self>,
        right: ClassInstanceRef<Self>,
    ) -> JvmResult<()> {
        tracing::debug!("m.A3::mul({this:?}, {left:?}, {right:?})");

        let left = Self::read(jvm, &left).await?;
        let right = Self::read(jvm, &right).await?;
        let mut product = [0i32; 12];
        for row in 0..3 {
            for column in 0..3 {
                let sum: i64 = (0..3).map(|index| left[row * 4 + index] as i64 * right[index * 4 + column] as i64).sum();
                product[row * 4 + column] = (sum / ONE) as i32;
            }
            let carried: i64 = (0..3).map(|index| left[row * 4 + index] as i64 * right[index * 4 + 3] as i64 / ONE).sum();
            product[row * 4 + 3] = (left[row * 4 + 3] as i64 + carried) as i32;
        }

        Self::write(jvm, &this, product).await
    }

    /// Applies the transform to a point: the rotation is fixed point, the translation in the
    /// coordinate's own units.
    async fn trans(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        destination: ClassInstanceRef<V3>,
        source: ClassInstanceRef<V3>,
    ) -> JvmResult<()> {
        tracing::debug!("m.A3::trans({this:?}, {destination:?}, {source:?})");

        let cells = Self::read(jvm, &this).await?;
        let [x, y, z] = V3::read(jvm, &source).await?;
        let mut out = [0i32; 3];
        for (row, value) in out.iter_mut().enumerate() {
            let sum = cells[row * 4] as i64 * x as i64 + cells[row * 4 + 1] as i64 * y as i64 + cells[row * 4 + 2] as i64 * z as i64;
            *value = (sum / ONE) as i32 + cells[row * 4 + 3];
        }

        V3::write(jvm, &destination, out).await
    }

    pub async fn read(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<[i32; 12]> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "transform is null").await);
        }
        let mut cells = [0i32; 12];
        for (cell, name) in cells.iter_mut().zip(CELLS) {
            *cell = jvm.get_field(this, "m/A3", name, "I").await?;
        }

        Ok(cells)
    }

    pub async fn write(jvm: &Jvm, this: &ClassInstanceRef<Self>, cells: [i32; 12]) -> JvmResult<()> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "transform is null").await);
        }
        let mut this = this.clone();
        for (name, value) in CELLS.iter().zip(cells) {
            jvm.put_field(&mut this, "m/A3", name, "I", value).await?;
        }

        Ok(())
    }
}
