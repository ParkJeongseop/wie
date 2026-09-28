use alloc::vec;

use alloc::string::String as RustString;

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxLength", "(I)V", Self::set_max_length, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("m_cPos", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("iMode", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("maxLength", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("m_td", "[C", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("imHandler", "Lorg/kwis/msp/lcdui/InputMethodHandler;", FieldAccessFlags::PROTECTED),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<TextComponent>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.TextComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        // TODO constant. 0: CONSTRAINT_ANY
        let im_handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (0,)).await?;

        jvm.put_field(&mut this, "imHandler", "Lorg/kwis/msp/lcdui/InputMethodHandler;", im_handler)
            .await?;

        Ok(())
    }

    async fn set_max_length(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<TextComponent>, max_length: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setMaxLength({this:?}, {max_length})");

        jvm.put_field(&mut this, "maxLength", "I", max_length).await?;

        Ok(())
    }

    async fn set_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<TextComponent>,
        data: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setString({this:?}, {data:?})");

        let chars = if data.is_null() {
            alloc::vec::Vec::new()
        } else {
            JavaLangString::to_rust_string(jvm, &data)
                .await?
                .encode_utf16()
                .collect::<alloc::vec::Vec<JavaChar>>()
        };
        let mut array: ClassInstanceRef<Array<JavaChar>> = jvm.instantiate_array("C", chars.len()).await?.into();
        jvm.store_array(&mut array, 0, chars).await?;
        jvm.put_field(&mut this, "m_td", "[C", array).await?;

        Ok(())
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getString({this:?})");

        let array: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&this, "m_td", "[C").await?;
        let text = if array.is_null() {
            RustString::new()
        } else {
            let length = jvm.array_length(&array).await?;
            RustString::from_utf16_lossy(&jvm.load_array(&array, 0, length).await?)
        };

        Ok(JavaLangString::from_rust_string(jvm, &text).await?.into())
    }
}
