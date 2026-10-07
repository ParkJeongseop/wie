use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.io.OutputConnection
pub struct OutputConnection;

impl OutputConnection {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/OutputConnection",
            parent_class: None,
            interfaces: vec!["javax/microedition/io/Connection"],
            methods: vec![
                JavaMethodProto::new_abstract(
                    "openOutputStream",
                    "()Ljava/io/OutputStream;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "openDataOutputStream",
                    "()Ljava/io/DataOutputStream;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
