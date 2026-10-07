use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.io.InputConnection
pub struct InputConnection;

impl InputConnection {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/InputConnection",
            parent_class: None,
            interfaces: vec!["javax/microedition/io/Connection"],
            methods: vec![
                JavaMethodProto::new_abstract(
                    "openInputStream",
                    "()Ljava/io/InputStream;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "openDataInputStream",
                    "()Ljava/io/DataInputStream;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
