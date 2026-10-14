use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class wec.SYSTheme: LG's theme store, where a title offers a sound as a ringtone. The handset's
// store is not here, so saving an item is refused. 월드장기체스 links the class at load.
pub struct SYSTheme;

impl SYSTheme {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/SYSTheme",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("saveItem", "(IILjava/lang/String;[BI)I", Self::save_item, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "FORMAT_SOUND_MA3",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "ITEM_GROUP_SOUND_DEFAULT",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("wec.SYSTheme::<clinit>");

        // The values are not documented; a title only hands them back to saveItem.
        jvm.put_static_field("wec/SYSTheme", "FORMAT_SOUND_MA3", "I", 1).await?;
        jvm.put_static_field("wec/SYSTheme", "ITEM_GROUP_SOUND_DEFAULT", "I", 0).await?;

        Ok(())
    }

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("wec.SYSTheme::<init>({this:?})");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn save_item(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        format: i32,
        group: i32,
        name: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub wec.SYSTheme::saveItem({this:?}, {format}, {group}, {name:?}, {data:?}, {length})");

        Ok(-1)
    }
}
