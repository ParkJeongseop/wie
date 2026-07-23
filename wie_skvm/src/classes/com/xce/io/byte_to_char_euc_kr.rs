use alloc::{vec, vec::Vec};

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.io.ByteToCharEUC_KR
#[allow(non_camel_case_types)]
pub struct ByteToCharEUC_KR;

impl ByteToCharEUC_KR {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/ByteToCharEUC_KR",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("convert", "([BII[CII)I", Self::convert, Default::default()),
                JavaMethodProto::new(
                    "getCharacterEncoding",
                    "()Ljava/lang/String;",
                    Self::get_character_encoding,
                    Default::default(),
                ),
                JavaMethodProto::new("getMaxCharsPerByte", "()I", Self::get_max_chars_per_byte, Default::default()),
                JavaMethodProto::new("flush", "([CII)I", Self::flush, Default::default()),
                JavaMethodProto::new("reset", "()V", Self::reset, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn convert(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        input: ClassInstanceRef<Array<i8>>,
        in_start: i32,
        in_end: i32,
        output: ClassInstanceRef<Array<JavaChar>>,
        out_start: i32,
        out_end: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::convert({this:?}, {in_start}, {in_end}, {out_start}, {out_end})");

        let length = (in_end - in_start).max(0);
        let raw: Vec<i8> = jvm.load_array(&input, in_start as _, length as _).await?;
        let bytes: Vec<u8> = raw.into_iter().map(|b| b as u8).collect();

        let decoded = encoding_rs::EUC_KR.decode(&bytes).0;
        let mut units: Vec<JavaChar> = decoded.encode_utf16().collect();

        let capacity = (out_end - out_start).max(0) as usize;
        if units.len() > capacity {
            tracing::warn!(
                "com.xce.io.ByteToCharEUC_KR::convert: output buffer too small ({} > {capacity})",
                units.len()
            );
            units.truncate(capacity);
        }

        let count = units.len() as i32;
        let mut output = output;
        jvm.store_array(&mut output, out_start as _, units).await?;

        Ok(count)
    }

    async fn get_character_encoding(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::getCharacterEncoding({this:?})");

        Ok(JavaLangString::from_rust_string(jvm, "EUC_KR").await?.into())
    }

    async fn get_max_chars_per_byte(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::getMaxCharsPerByte({this:?})");

        Ok(1)
    }

    async fn flush(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        _output: ClassInstanceRef<Array<JavaChar>>,
        _out_start: i32,
        _out_end: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::flush({this:?})");

        Ok(0)
    }

    async fn reset(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::reset({this:?})");

        Ok(())
    }
}
