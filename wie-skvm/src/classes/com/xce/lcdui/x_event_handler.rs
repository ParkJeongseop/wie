use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.lcdui.XEventHandler
//
// The vendor's event dispatcher. Titles reach the one instance through the static `eventHandler` and
// call `restoreDisplay` after the platform input method has drawn over the screen, to get their own
// frame back; nothing here draws over a title, so the restore is a redraw request.
pub struct XEventHandler;

impl XEventHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XEventHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("restoreDisplay", "()V", Self::restore_display, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new(
                "eventHandler",
                "Lcom/xce/lcdui/XEventHandler;",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
            )],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XEventHandler::<clinit>");

        let handler = jvm.new_class("com/xce/lcdui/XEventHandler", "()V", ()).await?;
        jvm.put_static_field("com/xce/lcdui/XEventHandler", "eventHandler", "Lcom/xce/lcdui/XEventHandler;", handler)
            .await
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XEventHandler::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn restore_display(_jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XEventHandler::restoreDisplay({this:?})");

        context.system().platform().screen().request_redraw().unwrap();

        Ok(())
    }
}
