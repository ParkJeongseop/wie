use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.ChoiceText: a component that shows one of a list of strings and lets the user
// pick another (미니고치). The shell it is shown in draws it and changes the selection
// (`net.wie.ShellCard`). Members beyond these are added as titles are seen to call them.
pub struct ChoiceText;

impl ChoiceText {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/ChoiceText",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "([Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getAllText", "()[Ljava/lang/String;", Self::get_all_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSelectedIndex", "()I", Self::get_selected_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setSelectedIndex", "(I)Z", Self::set_selected_index, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("text", "[Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("selected", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.ChoiceText::<init>({this:?}, {text:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "com/ktf/kfc/ChoiceText", "text", "[Ljava/lang/String;", text)
            .await?;

        Ok(())
    }

    async fn get_all_text(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
    ) -> JvmResult<ClassInstanceRef<Array<ClassInstanceRef<String>>>> {
        tracing::debug!("com.ktf.kfc.ChoiceText::getAllText({this:?})");

        jvm.get_field(&this, "com/ktf/kfc/ChoiceText", "text", "[Ljava/lang/String;").await
    }

    async fn get_selected_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.ktf.kfc.ChoiceText::getSelectedIndex({this:?})");

        jvm.get_field(&this, "com/ktf/kfc/ChoiceText", "selected", "I").await
    }

    async fn set_selected_index(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<bool> {
        tracing::debug!("com.ktf.kfc.ChoiceText::setSelectedIndex({this:?}, {index})");

        let text: ClassInstanceRef<Array<ClassInstanceRef<String>>> =
            jvm.get_field(&this, "com/ktf/kfc/ChoiceText", "text", "[Ljava/lang/String;").await?;
        if text.is_null() || index < 0 || index as usize >= jvm.array_length(&text).await? {
            return Ok(false);
        }
        jvm.put_field(&mut this, "com/ktf/kfc/ChoiceText", "selected", "I", index).await?;

        Ok(true)
    }
}
