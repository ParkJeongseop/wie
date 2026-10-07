use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.io.HttpConnection
pub struct HttpConnection;

impl HttpConnection {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/HttpConnection",
            parent_class: None,
            interfaces: vec!["javax/microedition/io/ContentConnection"],
            methods: vec![
                JavaMethodProto::new_abstract("getURL", "()Ljava/lang/String;", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getProtocol",
                    "()Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("getHost", "()Ljava/lang/String;", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getFile", "()Ljava/lang/String;", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getRef", "()Ljava/lang/String;", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getQuery",
                    "()Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("getPort", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getRequestMethod",
                    "()Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "setRequestMethod",
                    "(Ljava/lang/String;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getRequestProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "setRequestProperty",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("getResponseCode", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getResponseMessage",
                    "()Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("getExpiration", "()J", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getDate", "()J", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getLastModified", "()J", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getHeaderField",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getHeaderFieldInt",
                    "(Ljava/lang/String;I)I",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getHeaderFieldDate",
                    "(Ljava/lang/String;J)J",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getHeaderField",
                    "(I)Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract(
                    "getHeaderFieldKey",
                    "(I)Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
