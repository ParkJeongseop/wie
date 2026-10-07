use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const CONTENTS_MANAGER: &str = "mmpp/phone/ContentsManager";

// class mmpp.phone.ContentsManager
//
// Saves a title's ringtone or wallpaper into the handset's own contents. The host has no such
// store, so nothing is kept. The result codes are non-final statics the API does not number; titles
// compare against the fields, so any distinct values serve.
pub struct ContentsManager;

impl ContentsManager {
    const SUCCESS: i32 = 0;
    const CONTENTS_FULL: i32 = 1;
    const FAILURE: i32 = 2;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: CONTENTS_MANAGER,
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setCurrentMelody",
                    "(Ljava/lang/String;[BII)I",
                    Self::set_current_melody,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setCurrentDesktop",
                    "(Ljava/lang/String;[BII)I",
                    Self::set_current_desktop,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getNumberOfMelody",
                    "()I",
                    Self::get_number_of_contents,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getNumberOfDesktop",
                    "()I",
                    Self::get_number_of_contents,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("SUCCESS", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("CONTENTS_FULL", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("FAILURE", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field(CONTENTS_MANAGER, "SUCCESS", "I", Self::SUCCESS).await?;
        jvm.put_static_field(CONTENTS_MANAGER, "CONTENTS_FULL", "I", Self::CONTENTS_FULL).await?;
        jvm.put_static_field(CONTENTS_MANAGER, "FAILURE", "I", Self::FAILURE).await
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn set_current_melody(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        contents: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        tracing::warn!("stub mmpp.phone.ContentsManager::setCurrentMelody({name}, {contents:?}, {offset}, {length})");

        Ok(Self::SUCCESS)
    }

    async fn set_current_desktop(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        contents: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        tracing::warn!("stub mmpp.phone.ContentsManager::setCurrentDesktop({name}, {contents:?}, {offset}, {length})");

        Ok(Self::SUCCESS)
    }

    async fn get_number_of_contents(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        Ok(0)
    }
}
