use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.io.ContentConnection
pub struct ContentConnection;

impl ContentConnection {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/ContentConnection",
            parent_class: None,
            interfaces: vec!["javax/microedition/io/StreamConnection"],
            methods: vec![
                JavaMethodProto::new_abstract("getType", "()Ljava/lang/String;", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getEncoding",
                    "()Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("getLength", "()J", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
