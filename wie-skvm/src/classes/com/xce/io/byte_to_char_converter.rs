use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.io.ByteToCharConverter
//
// The java.io converter API of the day, which the vendor exposed: a title declares a field of this
// type, puts a ByteToCharEUC_KR in it and decodes its own Korean text through `convert` and `flush`
// into a char array it already owns, instead of going through String.
pub struct ByteToCharConverter;

impl ByteToCharConverter {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/ByteToCharConverter",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new_abstract("convert", "([BII[CII)I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("flush", "([CII)I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "getCharacterEncoding",
                    "()Ljava/lang/String;",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("getMaxCharsPerByte", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("reset", "()V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.ByteToCharConverter::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }
}
