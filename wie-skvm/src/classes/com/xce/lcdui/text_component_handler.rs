use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::{
    classes::com::xce::lcdui::TextComponent,
    ime::{Automata, Korean, Mode, Op},
};

// class com.xce.lcdui.TextComponentHandler
//
// The platform's keypad input method. A title gets the one handler through the static, registers the
// TextComponent it implements, forwards its key presses, and receives the composed characters through
// the component's insert/replace/delete. The automata (`crate::ime`) lives in the fields below so that
// a half-typed syllable is ordinary object state.
pub struct TextComponentHandler;

impl TextComponentHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponentHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new(
                    "getTextComponentHandler",
                    "()Lcom/xce/lcdui/TextComponentHandler;",
                    Self::get_text_component_handler,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("isLoaded", "()Z", Self::is_loaded, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "setTextComponent",
                    "(Lcom/xce/lcdui/TextComponent;)V",
                    Self::set_text_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getTextComponent",
                    "()Lcom/xce/lcdui/TextComponent;",
                    Self::get_text_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getInputMode", "()I", Self::get_input_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clear", "()V", Self::clear, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)Z", Self::key_pressed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)Z", Self::key_released, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyRepeated", "(I)Z", Self::key_repeated, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "handler",
                    "Lcom/xce/lcdui/TextComponentHandler;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
                JavaFieldProto::new("component", "Lcom/xce/lcdui/TextComponent;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("mode", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("composingKey", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("composingIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("choseong", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("jungseong", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("jongseong", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("composing", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lastKey", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lastIndex", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        // Korean titles come up composing Hangul; the field shows no indicator until a mode is read.
        Self::store(jvm, &mut this, &Automata::new(Mode::Korean)).await
    }

    async fn get_text_component_handler(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::getTextComponentHandler()");

        let handler: ClassInstanceRef<Self> = jvm
            .get_static_field("com/xce/lcdui/TextComponentHandler", "handler", "Lcom/xce/lcdui/TextComponentHandler;")
            .await?;
        if !handler.is_null() {
            return Ok(handler);
        }

        let handler: ClassInstanceRef<Self> = jvm.new_class("com/xce/lcdui/TextComponentHandler", "()V", ()).await?.into();
        jvm.put_static_field(
            "com/xce/lcdui/TextComponentHandler",
            "handler",
            "Lcom/xce/lcdui/TextComponentHandler;",
            handler.clone(),
        )
        .await?;

        Ok(handler)
    }

    async fn is_loaded(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::isLoaded()");

        Ok(true)
    }

    async fn set_text_component(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<TextComponent>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::setTextComponent({this:?}, {component:?})");

        jvm.put_field(&mut this, "component", "Lcom/xce/lcdui/TextComponent;", component).await?;

        let mut automata = Self::load(jvm, &this).await?;
        automata.commit();
        Self::store(jvm, &mut this, &automata).await
    }

    async fn get_text_component(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<TextComponent>> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::getTextComponent({this:?})");

        jvm.get_field(&this, "component", "Lcom/xce/lcdui/TextComponent;").await
    }

    async fn get_input_mode(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::getInputMode({this:?})");

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;

        Ok(Mode::from_index(mode).vendor_value())
    }

    async fn clear(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::clear({this:?})");

        let mut automata = Self::load(jvm, &this).await?;
        automata.commit();
        Self::store(jvm, &mut this, &automata).await
    }

    async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::keyPressed({this:?}, {key})");

        let component: ClassInstanceRef<TextComponent> = jvm.get_field(&this, "component", "Lcom/xce/lcdui/TextComponent;").await?;
        if component.is_null() {
            // Nothing to compose into: the key is the title's.
            return Ok(false);
        }

        let mut automata = Self::load(jvm, &this).await?;
        let (ops, handled) = automata.press(key);
        Self::store(jvm, &mut this, &automata).await?;

        for op in ops {
            match op {
                Op::Insert(c) => {
                    jvm.invoke_virtual(&component, "com/xce/lcdui/TextComponent", "insert", "(C)V", (c,))
                        .await?
                }
                Op::Replace(c) => {
                    jvm.invoke_virtual(&component, "com/xce/lcdui/TextComponent", "replace", "(C)V", (c,))
                        .await?
                }
                Op::Delete => jvm.invoke_virtual(&component, "com/xce/lcdui/TextComponent", "delete", "()V", ()).await?,
            }
        }

        Ok(handled)
    }

    // Composition happens on the press. A release is reported untaken, so a title that acts on
    // releases still can; a repeat is swallowed, so a held key does not spray glyphs.
    async fn key_released(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::keyReleased({this:?}, {key})");

        Ok(false)
    }

    async fn key_repeated(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::keyRepeated({this:?}, {key})");

        Ok(false)
    }

    async fn load(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Automata> {
        let mode: i32 = jvm.get_field(this, "mode", "I").await?;

        Ok(Automata {
            mode: Mode::from_index(mode),
            composing_key: jvm.get_field(this, "composingKey", "I").await?,
            composing_index: jvm.get_field(this, "composingIndex", "I").await?,
            korean: Korean {
                choseong: jvm.get_field(this, "choseong", "I").await?,
                jungseong: jvm.get_field(this, "jungseong", "I").await?,
                jongseong: jvm.get_field(this, "jongseong", "I").await?,
                composing: jvm.get_field(this, "composing", "Z").await?,
                last_key: jvm.get_field(this, "lastKey", "I").await?,
                last_index: jvm.get_field(this, "lastIndex", "I").await?,
            },
        })
    }

    async fn store(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, automata: &Automata) -> JvmResult<()> {
        jvm.put_field(this, "mode", "I", automata.mode.index()).await?;
        jvm.put_field(this, "composingKey", "I", automata.composing_key).await?;
        jvm.put_field(this, "composingIndex", "I", automata.composing_index).await?;
        jvm.put_field(this, "choseong", "I", automata.korean.choseong).await?;
        jvm.put_field(this, "jungseong", "I", automata.korean.jungseong).await?;
        jvm.put_field(this, "jongseong", "I", automata.korean.jongseong).await?;
        jvm.put_field(this, "composing", "Z", automata.korean.composing).await?;
        jvm.put_field(this, "lastKey", "I", automata.korean.last_key).await?;
        jvm.put_field(this, "lastIndex", "I", automata.korean.last_index).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

    use crate::classes::com::xce::lcdui::{TextComponent, TextComponentHandler};

    // A title's text component: keeps the text in a StringBuffer the way the callbacks describe.
    struct BufferComponent;

    impl BufferComponent {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/BufferComponent",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["com/xce/lcdui/TextComponent"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("insert", "(C)V", Self::insert, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("replace", "(C)V", Self::replace, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("delete", "()V", Self::delete, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("text", "()Ljava/lang/String;", Self::text, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![JavaFieldProto::new("buffer", "Ljava/lang/StringBuffer;", FieldAccessFlags::PRIVATE)],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
            let buffer = jvm.new_class("java/lang/StringBuffer", "()V", ()).await?;
            jvm.put_field(&mut this, "buffer", "Ljava/lang/StringBuffer;", buffer).await
        }

        async fn insert(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: JavaChar) -> JvmResult<()> {
            let buffer = jvm.get_field(&this, "buffer", "Ljava/lang/StringBuffer;").await?;
            let _: ClassInstanceRef<()> = jvm
                .invoke_virtual(&buffer, "java/lang/StringBuffer", "append", "(C)Ljava/lang/StringBuffer;", (c,))
                .await?;
            Ok(())
        }

        async fn replace(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: JavaChar) -> JvmResult<()> {
            Self::delete(jvm, context, this.clone()).await?;
            Self::insert(jvm, context, this, c).await
        }

        async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let buffer = jvm.get_field(&this, "buffer", "Ljava/lang/StringBuffer;").await?;
            let length: i32 = jvm.invoke_virtual(&buffer, "java/lang/StringBuffer", "length", "()I", ()).await?;
            let _: () = jvm
                .invoke_virtual(&buffer, "java/lang/StringBuffer", "setLength", "(I)V", ((length - 1).max(0),))
                .await?;
            Ok(())
        }

        async fn text(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
            let buffer = jvm.get_field(&this, "buffer", "Ljava/lang/StringBuffer;").await?;
            jvm.invoke_virtual(&buffer, "java/lang/StringBuffer", "toString", "()Ljava/lang/String;", ())
                .await
        }
    }

    #[test]
    fn keys_compose_into_the_registered_component() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [TextComponent::as_proto(), TextComponentHandler::as_proto(), BufferComponent::as_proto()].into(),
            ]),
            |jvm| async move {
                let handler: ClassInstanceRef<TextComponentHandler> = jvm
                    .invoke_static(
                        "com/xce/lcdui/TextComponentHandler",
                        "getTextComponentHandler",
                        "()Lcom/xce/lcdui/TextComponentHandler;",
                        (),
                    )
                    .await?;
                let again: ClassInstanceRef<TextComponentHandler> = jvm
                    .invoke_static(
                        "com/xce/lcdui/TextComponentHandler",
                        "getTextComponentHandler",
                        "()Lcom/xce/lcdui/TextComponentHandler;",
                        (),
                    )
                    .await?;
                assert_eq!(handler.identity(), again.identity());

                // Without a component the key is the title's.
                let handled: bool = jvm
                    .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "keyPressed", "(I)Z", ('4' as i32,))
                    .await?;
                assert!(!handled);

                let component = jvm.new_class("test/BufferComponent", "()V", ()).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &handler,
                        "com/xce/lcdui/TextComponentHandler",
                        "setTextComponent",
                        "(Lcom/xce/lcdui/TextComponent;)V",
                        (component.clone(),),
                    )
                    .await?;
                let mode: i32 = jvm
                    .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "getInputMode", "()I", ())
                    .await?;
                assert_eq!(mode, 16);

                for key in "4120".chars() {
                    let handled: bool = jvm
                        .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "keyPressed", "(I)Z", (key as i32,))
                        .await?;
                    assert!(handled);
                }
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&component, "test/BufferComponent", "text", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "강");

                // `*` switches to lower-case English and reports it; `#` is a space.
                for key in ['*', '#', '2', '2'] {
                    let _: bool = jvm
                        .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "keyPressed", "(I)Z", (key as i32,))
                        .await?;
                }
                let mode: i32 = jvm
                    .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "getInputMode", "()I", ())
                    .await?;
                assert_eq!(mode, 2);
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&component, "test/BufferComponent", "text", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "강 b");

                let released: bool = jvm
                    .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "keyReleased", "(I)Z", ('2' as i32,))
                    .await?;
                assert!(!released);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
