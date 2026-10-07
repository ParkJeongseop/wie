use alloc::vec;

use jvm_types::ClassAccessFlags;

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.io.StreamConnection
pub struct StreamConnection;

impl StreamConnection {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/StreamConnection",
            parent_class: None,
            interfaces: vec!["javax/microedition/io/InputConnection", "javax/microedition/io/OutputConnection"],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
