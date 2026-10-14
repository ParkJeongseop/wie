use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.net.Socket
//
// The vendor's socket. The titles only touch its PPP session controls — how long the dial-up link is
// kept after the last socket closes, and closing it — before and after their network menus; there is
// no link here to keep or close.
pub struct Socket;

impl Socket {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/net/Socket",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "setPPPPreserveTime",
                    "(I)V",
                    Self::set_ppp_preserve_time,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("PPPClose", "()V", Self::ppp_close, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn set_ppp_preserve_time(_jvm: &Jvm, _context: &mut WieJvmContext, seconds: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.net.Socket::setPPPPreserveTime({seconds})");

        Ok(())
    }

    async fn ppp_close(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.net.Socket::PPPClose()");

        Ok(())
    }
}
