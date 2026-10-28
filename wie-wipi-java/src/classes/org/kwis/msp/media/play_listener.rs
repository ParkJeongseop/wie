use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.media.PlayListener: `playUpdate(Clip clip, int event, int parm)` tells a
// title what its clip did. Events: ERROR -1, END_OF_DATA 1, START 2, STOP 3, PAUSE 4, RESUME 5,
// RECORD 6, FULL_OF_DATA 7.
pub struct PlayListener;

impl PlayListener {
    pub const END_OF_DATA: i32 = 1;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/PlayListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract(
                "playUpdate",
                "(Lorg/kwis/msp/media/Clip;II)V",
                MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
