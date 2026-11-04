use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.media.control.VolumeControl
pub struct VolumeControl;

impl VolumeControl {
    pub fn as_proto() -> WieJavaClassProto {
        let abstract_method = MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT;
        WieJavaClassProto {
            name: "javax/microedition/media/control/VolumeControl",
            parent_class: None,
            interfaces: vec!["javax/microedition/media/Control"],
            methods: vec![
                JavaMethodProto::new_abstract("setMute", "(Z)V", abstract_method),
                JavaMethodProto::new_abstract("isMuted", "()Z", abstract_method),
                JavaMethodProto::new_abstract("setLevel", "(I)I", abstract_method),
                JavaMethodProto::new_abstract("getLevel", "()I", abstract_method),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
