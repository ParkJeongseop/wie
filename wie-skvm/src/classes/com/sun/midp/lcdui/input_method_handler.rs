use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.sun.midp.lcdui.InputMethodHandler
//
// The reference implementation's input-method switch, which the SK-VM left reachable: a title reads
// the current mode to draw its indicator and sets the one it wants. The mode is a number the title
// chooses and reads back; nothing else interprets it.
pub struct InputMethodHandler;

impl InputMethodHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/sun/midp/lcdui/InputMethodHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new(
                    "getInputMethodHandler",
                    "()Lcom/sun/midp/lcdui/InputMethodHandler;",
                    Self::get_input_method_handler,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getInputMode", "()I", Self::get_input_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("switchInputMode", "(I)V", Self::switch_input_mode, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "handler",
                    "Lcom/sun/midp/lcdui/InputMethodHandler;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
                JavaFieldProto::new("mode", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.sun.midp.lcdui.InputMethodHandler::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "mode", "I", 1).await
    }

    async fn get_input_method_handler(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.sun.midp.lcdui.InputMethodHandler::getInputMethodHandler()");

        let handler: ClassInstanceRef<Self> = jvm
            .get_static_field(
                "com/sun/midp/lcdui/InputMethodHandler",
                "handler",
                "Lcom/sun/midp/lcdui/InputMethodHandler;",
            )
            .await?;
        if !handler.is_null() {
            return Ok(handler);
        }

        let handler: ClassInstanceRef<Self> = jvm.new_class("com/sun/midp/lcdui/InputMethodHandler", "()V", ()).await?.into();
        jvm.put_static_field(
            "com/sun/midp/lcdui/InputMethodHandler",
            "handler",
            "Lcom/sun/midp/lcdui/InputMethodHandler;",
            handler.clone(),
        )
        .await?;

        Ok(handler)
    }

    async fn get_input_mode(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.sun.midp.lcdui.InputMethodHandler::getInputMode({this:?})");

        jvm.get_field(&this, "mode", "I").await
    }

    async fn switch_input_mode(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, mode: i32) -> JvmResult<()> {
        tracing::debug!("com.sun.midp.lcdui.InputMethodHandler::switchInputMode({this:?}, {mode})");

        jvm.put_field(&mut this, "mode", "I", mode).await
    }
}
