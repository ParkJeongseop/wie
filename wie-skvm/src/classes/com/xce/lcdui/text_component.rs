use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface com.xce.lcdui.TextComponent
//
// The vendor's own text component: a title that keeps the text itself implements this and hands the
// object to TextComponentHandler.setTextComponent, and the platform input method drives it through
// these callbacks. The members are the thirteen every implementation in the library carries.
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponent",
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new_abstract("getCaretPosition", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getConstraints", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getMaxSize", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("size", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("insert", "(C)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("delete", "()V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("clear", "()V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("replace", "(C)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("moveCursor", "(I)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("setCaretPosition", "(I)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("setCaretVisible", "(Z)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("repaint", "()V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("repaintIM", "()V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
