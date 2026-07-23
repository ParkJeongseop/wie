use alloc::{string::String as RustString, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_backend::canvas;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// Default font point size (SIZE_MEDIUM). WIPI/MIDP feature phones render roughly
// this size on a 240x320 screen; kept at the previous hardcoded value to avoid
// regressing games that only ever use the default font.
const DEFAULT_POINT_SIZE: i32 = 10;

// class javax.microedition.lcdui.Font
pub struct Font;

impl Font {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Font",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("stringWidth", "(Ljava/lang/String;)I", Self::string_width, Default::default()),
                JavaMethodProto::new("substringWidth", "(Ljava/lang/String;II)I", Self::substring_width, Default::default()),
                JavaMethodProto::new("charWidth", "(C)I", Self::char_width, Default::default()),
                JavaMethodProto::new("charsWidth", "([CII)I", Self::chars_width, Default::default()),
                JavaMethodProto::new(
                    "getFont",
                    "(III)Ljavax/microedition/lcdui/Font;",
                    Self::get_font,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Ljavax/microedition/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("FACE_SYSTEM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_MONOSPACE", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_PROPORTIONAL", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_BOLD", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_ITALIC", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_UNDERLINED", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_MEDIUM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_LARGE", "I", FieldAccessFlags::STATIC),
                // instance state: resolved point size in points
                JavaFieldProto::new("pointSize", "I", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Font::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_SYSTEM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_MONOSPACE", "I", 32).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_PROPORTIONAL", "I", 64)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_PLAIN", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_BOLD", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_ITALIC", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_UNDERLINED", "I", 4).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_MEDIUM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_SMALL", "I", 8).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_LARGE", "I", 16).await?;

        Ok(())
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Font::<init>({this:?})");

        jvm.put_field(&mut this, "pointSize", "I", DEFAULT_POINT_SIZE).await?;

        Ok(())
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::getHeight({this:?})");

        let point_size: i32 = jvm.get_field(&this, "pointSize", "I").await?;

        Ok((canvas::font_height(point_size as f32) + 0.5) as i32)
    }

    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("javax.microedition.lcdui.Font::getFont({face}, {style}, {size})");

        let mut instance: ClassInstanceRef<Font> = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?.into();
        jvm.put_field(&mut instance, "pointSize", "I", Self::size_to_point(size)).await?;

        Ok(instance)
    }

    async fn string_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::stringWidth({this:?}, {string:?})");

        let point_size: i32 = jvm.get_field(&this, "pointSize", "I").await?;
        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        Ok(canvas::string_width(&string, point_size as f32) as _)
    }

    async fn substring_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::substringWidth({this:?}, {string:?}, {offset}, {len})");

        let point_size: i32 = jvm.get_field(&this, "pointSize", "I").await?;
        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        Ok(canvas::string_width(&substring, point_size as f32) as _)
    }

    async fn char_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::charWidth({this:?}, {char})");

        let point_size: i32 = jvm.get_field(&this, "pointSize", "I").await?;
        let string = RustString::from_utf16(&[char]).unwrap();

        Ok(canvas::string_width(&string, point_size as f32) as _)
    }

    async fn chars_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::charsWidth({this:?}, {chars:?}, {offset}, {len})");

        let point_size: i32 = jvm.get_field(&this, "pointSize", "I").await?;
        let chars = jvm.load_array(&chars, offset as _, len as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        Ok(canvas::string_width(&string, point_size as f32) as _)
    }

    // SIZE_SMALL=8, SIZE_MEDIUM=0, SIZE_LARGE=16 -> resolved point size for neodgm on 240x320.
    fn size_to_point(size: i32) -> i32 {
        match size {
            8 => 8,   // SIZE_SMALL
            16 => 13, // SIZE_LARGE
            _ => DEFAULT_POINT_SIZE,
        }
    }
}
