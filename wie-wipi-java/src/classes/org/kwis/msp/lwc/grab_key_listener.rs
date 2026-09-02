use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::ClassAccessFlags;
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lwc.GrabKeyListener
pub struct GrabKeyListener;

impl GrabKeyListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/GrabKeyListener",
            parent_class: None,
            interfaces: vec![],
            // boolean grabKeyNotify(int type, int chr, Object obj)
            methods: vec![JavaMethodProto::new_abstract(
                "grabKeyNotify",
                "(IILjava/lang/Object;)Z",
                Default::default(),
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
