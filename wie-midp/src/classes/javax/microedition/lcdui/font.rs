use alloc::{string::String as RustString, vec};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::canvas::string_width;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// The point size the glyphs are drawn at and the line height, for SIZE_SMALL, SIZE_MEDIUM and
// SIZE_LARGE. WIPI/MIDP feature phones render SIZE_MEDIUM at roughly this size on a 240x320 screen.
const DEFAULT_SIZES: [(f32, i32); 3] = [(8.0, 10), (10.0, Font::HEIGHT), (13.0, 16)];

// class javax.microedition.lcdui.Font
pub struct Font;

impl Font {
    pub const HEIGHT: i32 = 12;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Font",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getBaselinePosition", "()I", Self::get_baseline_position, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSize", "()I", Self::get_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stringWidth", "(Ljava/lang/String;)I", Self::string_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "substringWidth",
                    "(Ljava/lang/String;II)I",
                    Self::substring_width,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("charWidth", "(C)I", Self::char_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("charsWidth", "([CII)I", Self::chars_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getFont",
                    "(III)Ljavax/microedition/lcdui/Font;",
                    Self::get_font,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Ljavax/microedition/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "FACE_SYSTEM",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "FACE_MONOSPACE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "FACE_PROPORTIONAL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_PLAIN",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_BOLD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_ITALIC",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_UNDERLINED",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIZE_SMALL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIZE_MEDIUM",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIZE_LARGE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "defaultFont",
                    "Ljavax/microedition/lcdui/Font;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
                // instance state: the point size the glyphs are drawn at and the line height
                JavaFieldProto::new("pointSize", "F", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
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

        let (point_size, height) = Self::sizes(jvm).await?[1];
        jvm.put_field(&mut this, "pointSize", "F", point_size).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;

        Ok(())
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::getHeight({this:?})");

        jvm.get_field(&this, "height", "I").await
    }

    async fn get_baseline_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::getBaselinePosition({this:?})");

        let height: i32 = jvm.get_field(&this, "height", "I").await?;

        // the bundled font keeps a sixth of its cell below the baseline
        Ok((height * 5 + 3) / 6)
    }

    async fn get_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::getSize({this:?})");

        let point_size: f32 = jvm.get_field(&this, "pointSize", "F").await?;
        let [(small, _), _, (large, _)] = Self::sizes(jvm).await?;

        // SIZE_SMALL=8, SIZE_MEDIUM=0, SIZE_LARGE=16
        Ok(if point_size == small {
            8
        } else if point_size == large {
            16
        } else {
            0
        })
    }

    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.lcdui.Font::getDefaultFont");

        // every Graphics starts out with it, so there is one instance
        let default_font: ClassInstanceRef<Self> = jvm
            .get_static_field("javax/microedition/lcdui/Font", "defaultFont", "Ljavax/microedition/lcdui/Font;")
            .await?;
        if !default_font.is_null() {
            return Ok(default_font);
        }

        let default_font: ClassInstanceRef<Self> = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?.into();
        jvm.put_static_field(
            "javax/microedition/lcdui/Font",
            "defaultFont",
            "Ljavax/microedition/lcdui/Font;",
            default_font.clone(),
        )
        .await?;

        Ok(default_font)
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("javax.microedition.lcdui.Font::getFont({face}, {style}, {size})");

        let [small, medium, large] = Self::sizes(jvm).await?;
        let (point_size, height) = match size {
            8 => small,
            16 => large,
            _ => medium,
        };

        let mut instance: ClassInstanceRef<Font> = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?.into();
        jvm.put_field(&mut instance, "pointSize", "F", point_size).await?;
        jvm.put_field(&mut instance, "height", "I", height).await?;

        Ok(instance)
    }

    async fn string_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::stringWidth({this:?}, {string:?})");

        let point_size: f32 = jvm.get_field(&this, "pointSize", "F").await?;
        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        Ok(string_width(context.system().platform().font(), &string, point_size) as _)
    }

    async fn substring_width(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::substringWidth({this:?}, {string:?}, {offset}, {len})");

        let point_size: f32 = jvm.get_field(&this, "pointSize", "F").await?;
        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        Ok(string_width(context.system().platform().font(), &substring, point_size) as _)
    }

    async fn char_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::charWidth({this:?}, {char})");

        let point_size: f32 = jvm.get_field(&this, "pointSize", "F").await?;
        let string = RustString::from_utf16(&[char]).unwrap();

        Ok(string_width(context.system().platform().font(), &string, point_size) as _)
    }

    async fn chars_width(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::charsWidth({this:?}, {chars:?}, {offset}, {len})");

        let point_size: f32 = jvm.get_field(&this, "pointSize", "F").await?;
        let chars = jvm.load_array(&chars, offset as _, len as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        Ok(string_width(context.system().platform().font(), &string, point_size) as _)
    }

    /// The point size and line height of SIZE_SMALL, SIZE_MEDIUM and SIZE_LARGE. A platform whose
    /// handsets set text in other sizes lists its own in the `wie.midp.font.sizes` system property
    /// as three comma-separated `points:height` entries.
    async fn sizes(jvm: &Jvm) -> JvmResult<[(f32, i32); 3]> {
        let key = JavaLangString::from_rust_string(jvm, "wie.midp.font.sizes").await?;
        let value: ClassInstanceRef<String> = jvm
            .invoke_static("java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", (key,))
            .await?;
        if value.is_null() {
            return Ok(DEFAULT_SIZES);
        }

        let value = JavaLangString::to_rust_string(jvm, &value).await?;
        let mut sizes = DEFAULT_SIZES;
        for (size, entry) in sizes.iter_mut().zip(value.split(',')) {
            if let Some((point_size, height)) = entry.split_once(':')
                && let (Ok(point_size), Ok(height)) = (point_size.trim().parse(), height.trim().parse())
            {
                *size = (point_size, height);
            }
        }

        Ok(sizes)
    }
}
