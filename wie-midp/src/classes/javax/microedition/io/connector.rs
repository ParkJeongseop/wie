use alloc::vec;

use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{
    io::{DataInputStream, DataOutputStream, InputStream, OutputStream},
    lang::String,
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::Connection;

// class javax.microedition.io.Connector
//
// The host gives applications no network: every open fails the way a handset without coverage
// does, with ConnectionNotFoundException (an IOException), which titles handle as "network error".
pub struct Connector;

impl Connector {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/Connector",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;)Ljavax/microedition/io/Connection;",
                    Self::open,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;I)Ljavax/microedition/io/Connection;",
                    Self::open_with_mode,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;IZ)Ljavax/microedition/io/Connection;",
                    Self::open_with_timeouts,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openInputStream",
                    "(Ljava/lang/String;)Ljava/io/InputStream;",
                    Self::open_input_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataInputStream",
                    "(Ljava/lang/String;)Ljava/io/DataInputStream;",
                    Self::open_data_input_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openOutputStream",
                    "(Ljava/lang/String;)Ljava/io/OutputStream;",
                    Self::open_output_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataOutputStream",
                    "(Ljava/lang/String;)Ljava/io/DataOutputStream;",
                    Self::open_data_output_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn unavailable(jvm: &Jvm, name: &ClassInstanceRef<String>) -> JavaError {
        let name = if name.is_null() {
            alloc::string::String::new()
        } else {
            match JavaLangString::to_rust_string(jvm, name).await {
                Ok(name) => name,
                Err(error) => return error,
            }
        };
        tracing::warn!("javax.microedition.io.Connector: no network, refusing {name}");

        jvm.exception("javax/microedition/io/ConnectionNotFoundException", &name).await
    }

    async fn open(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Connection>> {
        Err(Self::unavailable(jvm, &name).await)
    }

    async fn open_with_mode(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, _mode: i32) -> JvmResult<ClassInstanceRef<Connection>> {
        Err(Self::unavailable(jvm, &name).await)
    }

    async fn open_with_timeouts(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        _mode: i32,
        _timeouts: bool,
    ) -> JvmResult<ClassInstanceRef<Connection>> {
        Err(Self::unavailable(jvm, &name).await)
    }

    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<InputStream>> {
        Err(Self::unavailable(jvm, &name).await)
    }

    async fn open_data_input_stream(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<DataInputStream>> {
        Err(Self::unavailable(jvm, &name).await)
    }

    async fn open_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<OutputStream>> {
        Err(Self::unavailable(jvm, &name).await)
    }

    async fn open_data_output_stream(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<DataOutputStream>> {
        Err(Self::unavailable(jvm, &name).await)
    }
}
