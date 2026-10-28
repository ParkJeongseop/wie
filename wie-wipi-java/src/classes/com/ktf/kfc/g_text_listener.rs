use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::ktf::kfc::GTextField;

// class com.ktf.kfc.GTextListener: what a GTextField notifies of edits, and where a title sets
// the input modes the field offers. A title subclasses it or takes the field's own.
pub struct GTextListener;

impl GTextListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GTextListener",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setIMEModes", "([I)V", Self::set_ime_modes, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "textChanged",
                    "(Lcom/ktf/kfc/GTextField;)V",
                    Self::text_changed,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextListener::<init>({this:?})");

        Ok(())
    }

    async fn set_ime_modes(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, modes: ClassInstanceRef<Array<i32>>) -> JvmResult<()> {
        tracing::debug!("stub com.ktf.kfc.GTextListener::setIMEModes({this:?}, {modes:?})");

        Ok(())
    }

    async fn text_changed(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, field: ClassInstanceRef<GTextField>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextListener::textChanged({this:?}, {field:?})");

        Ok(())
    }
}
