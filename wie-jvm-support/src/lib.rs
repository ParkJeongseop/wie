#![no_std]
extern crate alloc;

mod context;
mod jvm_implementation;
pub mod native;
mod runtime;

use alloc::{
    boxed::Box,
    format,
    string::{String as RustString, ToString},
};

use jvm::{ClassInstanceRef, JavaError, Jvm, runtime::JavaLangString};
use rustjava_runtime::Runtime;
use rustjava_runtime::classes::java::lang::String;

use wie_backend::System;
use wie_util::{Result, WieError};

pub use context::{WieJavaClassProto, WieJvmContext};
pub use jvm_implementation::{JvmImplementation, RustJavaJvmImplementation};
use runtime::JvmRuntime;

pub static WIE_RUSTJAR: &str = "wie.rustjar";

pub struct JvmSupport;

impl JvmSupport {
    pub async fn new_jvm<T>(
        system: &System,
        jar_name: Option<&str>,
        protos: Box<[Box<[WieJavaClassProto]>]>,
        properties: &[(&str, &str)],
        implementation: T,
    ) -> Result<Jvm>
    where
        T: JvmImplementation + Sync + Send + 'static,
    {
        let runtime = JvmRuntime::new(system.clone(), implementation, protos);

        let class_path = if let Some(x) = jar_name {
            if cfg!(windows) {
                format!("{WIE_RUSTJAR};{x}")
            } else {
                format!("{WIE_RUSTJAR}:{x}")
            }
        } else {
            WIE_RUSTJAR.to_string()
        };

        let properties = [
            ("file.encoding", "EUC-KR"),
            ("java.class.path", &class_path),
            //("rustjava.disable_explicit_gc", "true"),
        ]
        .iter()
        .chain(properties.iter())
        .copied()
        .collect();
        let jvm = Jvm::new(
            rustjava_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await
        .map_err(|x| WieError::FatalError(format!("Failed to create JVM: {x}")))?;

        Ok(jvm)
    }

    pub async fn to_wie_err(jvm: &Jvm, err: JavaError) -> WieError {
        match err {
            JavaError::JavaException(x) => {
                // Name the exception before rendering it: printStackTrace on an app-defined
                // (AOT) exception class runs guest code, which can fail and hide the cause.
                let class_name = x.class_definition().name().into_owned();
                let message: Option<ClassInstanceRef<String>> = jvm
                    .invoke_virtual(&x, "java/lang/Throwable", "getMessage", "()Ljava/lang/String;", ())
                    .await
                    .ok();
                let message = match message {
                    Some(message) if !message.is_null() => JavaLangString::to_rust_string(jvm, &message).await.unwrap_or_default(),
                    _ => RustString::new(),
                };
                tracing::error!("Uncaught Java exception {class_name}: {message}");

                let trace = async {
                    let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await?;
                    let print_writer = jvm
                        .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(&x, "java/lang/Throwable", "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
                        .await?;
                    let trace = jvm
                        .invoke_virtual(&string_writer, "java/io/StringWriter", "toString", "()Ljava/lang/String;", [])
                        .await?;
                    JavaLangString::to_rust_string(jvm, &trace).await
                }
                .await;

                match trace {
                    Ok(trace) => WieError::FatalError(format!("\n{trace}")),
                    Err(_) => WieError::FatalError(format!("{class_name}: {message}")),
                }
            }
        }
    }
}
