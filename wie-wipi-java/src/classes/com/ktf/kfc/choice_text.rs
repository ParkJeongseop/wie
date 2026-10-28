use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.ChoiceText: a component that shows one of a list of strings (미니고치). Members
// beyond the constructor are added as titles are seen to call them.
pub struct ChoiceText;

impl ChoiceText {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/ChoiceText",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "([Ljava/lang/String;)V",
                Self::init,
                MethodAccessFlags::PUBLIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        choices: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.ChoiceText::<init>({this:?}, {choices:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }
}
