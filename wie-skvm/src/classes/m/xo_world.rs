use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::io::InputStream;
use wie_midp::classes::javax::microedition::lcdui::Graphics;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::{A3, V3, a3::IDENTITY, angle_to_radians, fixed};

// class m.XO_World
pub struct XOWorld;

impl XOWorld {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "m/XO_World",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("sin", "(I)I", Self::sin, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("cos", "(I)I", Self::cos, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "getViewTrans",
                    "(Lm/V3;Lm/V3;Lm/V3;Lm/A3;)V",
                    Self::get_view_trans,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("rotY", "(ILm/A3;)V", Self::rot_y, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("rotZ", "(ILm/A3;)V", Self::rot_z, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("loadMBAC", "(Ljava/io/InputStream;)I", Self::load_mbac, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loadMTRA", "(Ljava/io/InputStream;)I", Self::load_mtra, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loadBMP", "(Ljava/io/InputStream;)I", Self::load_bmp, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("shareData", "(Lm/XO_World;)V", Self::share_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setVram",
                    "(Ljavax/microedition/lcdui/Graphics;Lm/XO_World;II)V",
                    Self::set_vram,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setView", "(Lm/A3;IIII)V", Self::set_view, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setClip", "(IIII)V", Self::set_clip, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPosture", "(II)V", Self::set_posture, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxFrame", "(I)I", Self::get_max_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("draw", "(Ljavax/microedition/lcdui/Graphics;)V", Self::draw, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("dispose", "()V", Self::dispose, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn sin(_: &Jvm, _: &mut WieJvmContext, angle: i32) -> JvmResult<i32> {
        tracing::debug!("m.XO_World::sin({angle})");

        Ok(fixed(libm::sin(angle_to_radians(angle))))
    }

    async fn cos(_: &Jvm, _: &mut WieJvmContext, angle: i32) -> JvmResult<i32> {
        tracing::debug!("m.XO_World::cos({angle})");

        Ok(fixed(libm::cos(angle_to_radians(angle))))
    }

    /// The transform that takes a point into the camera's frame: the basis of a look-at from
    /// `position` towards `target`, with the camera's position carried through it.
    async fn get_view_trans(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        position: ClassInstanceRef<V3>,
        target: ClassInstanceRef<V3>,
        up: ClassInstanceRef<V3>,
        out: ClassInstanceRef<A3>,
    ) -> JvmResult<()> {
        tracing::debug!("m.XO_World::getViewTrans({position:?}, {target:?}, {up:?}, {out:?})");

        let eye = V3::read(jvm, &position).await?.map(|value| value as f64);
        let at = V3::read(jvm, &target).await?.map(|value| value as f64);
        let up = V3::read(jvm, &up).await?.map(|value| value as f64);

        let forward = normalize([at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]]);
        let side = normalize(cross(forward, up));
        let true_up = cross(side, forward);
        let rows = [side, true_up, [-forward[0], -forward[1], -forward[2]]];

        let mut cells = IDENTITY;
        for (row, basis) in rows.iter().enumerate() {
            for column in 0..3 {
                cells[row * 4 + column] = fixed(basis[column]);
            }
            cells[row * 4 + 3] = libm::round(-(basis[0] * eye[0] + basis[1] * eye[1] + basis[2] * eye[2])) as i32;
        }

        A3::write(jvm, &out, cells).await
    }

    async fn rot_y(jvm: &Jvm, _: &mut WieJvmContext, angle: i32, target: ClassInstanceRef<A3>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::rotY({angle}, {target:?})");

        let (sin, cos) = Self::sin_cos(angle);
        let mut cells = IDENTITY;
        (cells[0], cells[2], cells[8], cells[10]) = (cos, sin, -sin, cos);

        A3::write(jvm, &target, cells).await
    }

    async fn rot_z(jvm: &Jvm, _: &mut WieJvmContext, angle: i32, target: ClassInstanceRef<A3>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::rotZ({angle}, {target:?})");

        let (sin, cos) = Self::sin_cos(angle);
        let mut cells = IDENTITY;
        (cells[0], cells[1], cells[4], cells[5]) = (cos, -sin, sin, cos);

        A3::write(jvm, &target, cells).await
    }

    fn sin_cos(angle: i32) -> (i32, i32) {
        let radians = angle_to_radians(angle);

        (fixed(libm::sin(radians)), fixed(libm::cos(radians)))
    }

    // The renderer keeps no model: a loader answers a handle the title discards, and a model with
    // no frames is what an unloaded motion has.
    async fn load_mbac(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> JvmResult<i32> {
        tracing::warn!("stub m.XO_World::loadMBAC({this:?}, {stream:?})");

        Ok(0)
    }

    async fn load_mtra(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> JvmResult<i32> {
        tracing::warn!("stub m.XO_World::loadMTRA({this:?}, {stream:?})");

        Ok(0)
    }

    async fn load_bmp(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> JvmResult<i32> {
        tracing::warn!("stub m.XO_World::loadBMP({this:?}, {stream:?})");

        Ok(0)
    }

    async fn share_data(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, other: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::shareData({this:?}, {other:?})");

        Ok(())
    }

    async fn set_vram(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        other: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::setVram({this:?}, {graphics:?}, {other:?}, {x}, {y})");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn set_view(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        view: ClassInstanceRef<A3>,
        a: i32,
        b: i32,
        c: i32,
        d: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::setView({this:?}, {view:?}, {a}, {b}, {c}, {d})");

        Ok(())
    }

    async fn set_clip(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::setClip({this:?}, {x}, {y}, {width}, {height})");

        Ok(())
    }

    async fn set_posture(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, motion: i32, frame: i32) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::setPosture({this:?}, {motion}, {frame})");

        Ok(())
    }

    async fn get_max_frame(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, motion: i32) -> JvmResult<i32> {
        tracing::warn!("stub m.XO_World::getMaxFrame({this:?}, {motion})");

        Ok(0)
    }

    async fn draw(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::draw({this:?}, {graphics:?})");

        Ok(())
    }

    async fn dispose(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub m.XO_World::dispose({this:?})");

        Ok(())
    }
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let length = libm::sqrt(vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]);
    if length == 0.0 {
        return [0.0; 3];
    }

    vector.map(|value| value / length)
}
