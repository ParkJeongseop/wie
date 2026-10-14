use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.Message: a message framework datagram, an address and its bytes.
// 간호사타이쿤2 (LGT) links the constructor and getData at load and never sends one.
pub struct Message;

impl Message {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/Message",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;[B)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getAddress", "()Ljava/lang/String;", Self::get_address, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getData", "()[B", Self::get_data, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("address", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("data", "[B", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        address: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msf.io.Message::<init>({this:?}, {address:?}, {data:?})");

        jvm.put_field(&mut this, "address", "Ljava/lang/String;", address).await?;
        jvm.put_field(&mut this, "data", "[B", data).await?;

        Ok(())
    }

    async fn get_address(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msf.io.Message::getAddress({this:?})");

        jvm.get_field(&this, "address", "Ljava/lang/String;").await
    }

    async fn get_data(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("org.kwis.msf.io.Message::getData({this:?})");

        jvm.get_field(&this, "data", "[B").await
    }
}
