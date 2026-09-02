use alloc::{boxed::Box, vec};

use jvm::{ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaMethodProto, MethodBody};
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::{Class as JavaLangClass, String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::midlet::MIDlet;

// class net.wie.Launcher
pub struct Launcher;

impl Launcher {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/Launcher",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "start",
                    "(Ljava/lang/String;)V",
                    Self::start,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "startMIDlet",
                    "(Ljavax/microedition/midlet/MIDlet;)V",
                    Self::start_midlet,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn start(jvm: &Jvm, _context: &mut WieJvmContext, main_class: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::start({main_class:?})");

        // Load the MIDlet through the system class loader: `jvm.new_class` resolves via
        // the *current* class's loader, and net/wie itself lives in the runtime rustjar
        // loader whose chain cannot see the application jar on the class path.
        let class_loader = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
            .await?;
        let clazz: ClassInstanceRef<JavaLangClass> = jvm
            .invoke_virtual(
                &class_loader,
                "java/lang/ClassLoader",
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                (main_class,),
            )
            .await?;
        let main_class: Box<dyn jvm::ClassInstance> = jvm
            .invoke_virtual(&clazz, "java/lang/Class", "newInstance", "()Ljava/lang/Object;", ())
            .await?;

        jvm.invoke_static("net/wie/Launcher", "startMIDlet", "(Ljavax/microedition/midlet/MIDlet;)V", (main_class,))
            .await
    }

    async fn start_midlet(jvm: &Jvm, context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::startMIDlet({midlet:?})");

        // run startApp
        let _: () = jvm
            .invoke_virtual(&midlet, "javax/microedition/midlet/MIDlet", "startApp", "()V", ())
            .await?;

        // spawn event loop
        context.spawn(jvm, Box::new(EventLoopRunner))?;

        Ok(())
    }
}

struct EventLoopRunner;

#[async_trait::async_trait]
impl MethodBody<JavaError, WieJvmContext> for EventLoopRunner {
    async fn call(&self, jvm: &Jvm, _context: &mut WieJvmContext, _args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        jvm.attach_thread(None).await?;

        // event loop
        let event_queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;

        let event = jvm.instantiate_array("I", 4).await?;
        loop {
            let _: () = jvm
                .invoke_virtual(&event_queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                .await?;
            let _: () = jvm
                .invoke_virtual(&event_queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                .await?;
        }
    }
}
