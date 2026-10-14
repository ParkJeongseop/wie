use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class wec.DMInfo: what wec.OEMDevice.getDMInfo() answers. Nothing documents its members; 해적왕2007
// asks for the object at start and reported "플랫폼 예외" on the missing class. Members are added
// as titles are seen to call them.
pub struct DMInfo;

impl DMInfo {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/DMInfo",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getDMInfo",
                    "()Lwec/DMInfo;",
                    Self::get_dm_info,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("gethandsetMIN", "()Ljava/lang/String;", Self::get_handset_min, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn get_dm_info(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("wec.DMInfo::getDMInfo()");

        let info = jvm.new_class("wec/DMInfo", "()V", ()).await?;

        Ok(info.into())
    }

    /// The handset's MIN, the same answer `HandsetProperty.getProperty("MIN")` gives (empty here:
    /// a subscriber number is not something this platform makes up).
    async fn get_handset_min(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("wec.DMInfo::gethandsetMIN({this:?})");

        let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(jvm, "MIN").await?.into();
        jvm.invoke_static(
            "org/kwis/msp/handset/HandsetProperty",
            "getSystemProperty",
            "(Ljava/lang/String;)Ljava/lang/String;",
            (name,),
        )
        .await
    }

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("wec.DMInfo::<init>({this:?})");

        Ok(())
    }
}
