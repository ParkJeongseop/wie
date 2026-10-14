use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.jam.XBrowser
//
// The handset's application manager. Titles tell it which network mode their session wants before
// going online; nothing here goes online.
pub struct XBrowser;

impl XBrowser {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/jam/XBrowser",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "setNetworkMode",
                "(I)V",
                Self::set_network_mode,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn set_network_mode(_jvm: &Jvm, _context: &mut WieJvmContext, mode: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.jam.XBrowser::setNetworkMode({mode})");

        Ok(())
    }
}
