use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::ClassAccessFlags;
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lcdui.InputMethodListener
pub struct InputMethodListener;

impl InputMethodListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/InputMethodListener",
            parent_class: None,
            interfaces: vec![],
            // void notifyTextChanged(char[] chText, int length, int pMode)
            methods: vec![JavaMethodProto::new_abstract("notifyTextChanged", "([CII)V", Default::default())],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
