use alloc::{string::String as RustString, vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::{
    javax::microedition::lcdui::{Canvas, Font, Graphics},
    net::wie::{KeyMap, MIDPKeyCode},
};

const TEXT_FIELD_X: &str = "mmpp/microedition/lcdui/TextFieldX";
const GRAPHICS: &str = "javax/microedition/lcdui/Graphics";
const FONT: &str = "javax/microedition/lcdui/Font";

// class mmpp.microedition.lcdui.TextFieldX
//
// A text entry box a title paints inside its own Canvas and feeds key events to. Entry works in
// the numeric and the two roman modes (multi-tap: the same key again cycles its letters, the right
// key or any other key commits). Korean and symbol composition are not implemented, so
// nextInputMode cycles through the three working modes only. The caret is always at the end.
pub struct TextFieldX;

impl TextFieldX {
    const IM_NONE: i32 = 0;
    const IM_ROMAN_CAPS: i32 = 1;
    const IM_ROMAN_SMALL: i32 = 2;
    const IM_NUMERIC: i32 = 4;

    // javax.microedition.lcdui.TextField constraints
    const CONSTRAINT_MASK: i32 = 0xffff;
    const NUMERIC: i32 = 2;
    const PHONENUMBER: i32 = 3;
    const PASSWORD: i32 = 0x10000;

    const DEFAULT_WIDTH: i32 = 100;
    const PADDING: i32 = 2;
    const NO_KEY: i32 = i32::MIN;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: TEXT_FIELD_X,
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setOwner",
                    "(Ljavax/microedition/lcdui/Canvas;)V",
                    Self::set_owner,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getOwner",
                    "()Ljavax/microedition/lcdui/Canvas;",
                    Self::get_owner,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setWidth", "(I)V", Self::set_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxRow", "()I", Self::get_max_row, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxRow", "(I)V", Self::set_max_row, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFont", "()Ljavax/microedition/lcdui/Font;", Self::get_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFont", "(Ljavax/microedition/lcdui/Font;)V", Self::set_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextInputMode", "()I", Self::next_input_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getInputMode", "()I", Self::get_input_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getChars", "([C)I", Self::get_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setChars", "([CII)V", Self::set_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("insert", "(Ljava/lang/String;I)V", Self::insert, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("insert", "([CIII)V", Self::insert_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("delete", "(II)V", Self::delete, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxSize", "()I", Self::get_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxSize", "(I)I", Self::set_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCaretPosition", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getConstraints", "()I", Self::get_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setConstraints", "(I)V", Self::set_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFocus", "(Z)V", Self::set_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasFocus", "()Z", Self::has_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_released, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("label", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("constraints", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("owner", "Ljavax/microedition/lcdui/Canvas;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxRow", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("font", "Ljavax/microedition/lcdui/Font;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("focus", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("inputMode", "I", FieldAccessFlags::PRIVATE),
                // multi-tap state: the key whose letters are being cycled and the position in them
                JavaFieldProto::new("tapKey", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapIndex", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::<init>({this:?}, {label:?}, {text:?}, {max_size}, {constraints})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must be positive").await);
        }
        let text = if text.is_null() {
            RustString::new()
        } else {
            JavaLangString::to_rust_string(jvm, &text).await?
        };
        if text.chars().count() > max_size as usize {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "text longer than maxSize").await);
        }
        let font: ClassInstanceRef<Font> = jvm.invoke_static(FONT, "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ()).await?;

        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;
        jvm.put_field(&mut this, "constraints", "I", constraints).await?;
        jvm.put_field(&mut this, "width", "I", Self::DEFAULT_WIDTH).await?;
        jvm.put_field(&mut this, "maxRow", "I", 1).await?;
        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", font).await?;
        jvm.put_field(&mut this, "inputMode", "I", Self::first_input_mode(constraints)).await?;
        jvm.put_field(&mut this, "tapKey", "I", Self::NO_KEY).await?;

        Self::store_text(jvm, &mut this, &text).await
    }

    fn numeric_only(constraints: i32) -> bool {
        matches!(constraints & Self::CONSTRAINT_MASK, Self::NUMERIC | Self::PHONENUMBER)
    }

    fn first_input_mode(constraints: i32) -> i32 {
        if Self::numeric_only(constraints) {
            Self::IM_NUMERIC
        } else {
            Self::IM_ROMAN_SMALL
        }
    }

    async fn text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<RustString> {
        let text: ClassInstanceRef<String> = jvm.get_field(this, "text", "Ljava/lang/String;").await?;

        JavaLangString::to_rust_string(jvm, &text).await
    }

    async fn store_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: &str) -> JvmResult<()> {
        let text = JavaLangString::from_rust_string(jvm, text).await?;
        jvm.put_field(this, "text", "Ljava/lang/String;", text).await?;

        // the owner draws the field, so new contents need a repaint of the owner
        let owner: ClassInstanceRef<Canvas> = jvm.get_field(this, "owner", "Ljavax/microedition/lcdui/Canvas;").await?;
        if !owner.is_null() {
            let _: () = jvm
                .invoke_virtual(&owner, "javax/microedition/lcdui/Canvas", "repaint", "()V", ())
                .await?;
        }

        Ok(())
    }

    /// Replaces the contents after checking them against the size limit and the constraints.
    async fn replace_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: &str) -> JvmResult<()> {
        let max_size: i32 = jvm.get_field(this, "maxSize", "I").await?;
        let constraints: i32 = jvm.get_field(this, "constraints", "I").await?;

        if text.chars().count() > max_size as usize {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "text longer than maxSize").await);
        }
        if Self::numeric_only(constraints) && !text.chars().all(|x| x.is_ascii_digit() || "+-*#".contains(x)) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "text does not match the constraints")
                .await);
        }
        jvm.put_field(this, "tapKey", "I", Self::NO_KEY).await?;

        Self::store_text(jvm, this, text).await
    }

    async fn set_owner(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, owner: ClassInstanceRef<Canvas>) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::setOwner({this:?}, {owner:?})");

        jvm.put_field(&mut this, "owner", "Ljavax/microedition/lcdui/Canvas;", owner).await
    }

    async fn get_owner(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Canvas>> {
        jvm.get_field(&this, "owner", "Ljavax/microedition/lcdui/Canvas;").await
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "width", "I").await
    }

    async fn set_width(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::setWidth({this:?}, {width})");

        if width <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "width must be positive").await);
        }

        jvm.put_field(&mut this, "width", "I", width).await
    }

    async fn line_height(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        let font: ClassInstanceRef<Font> = jvm.get_field(this, "font", "Ljavax/microedition/lcdui/Font;").await?;

        jvm.invoke_virtual(&font, FONT, "getHeight", "()I", ()).await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let max_row: i32 = jvm.get_field(&this, "maxRow", "I").await?;

        Ok(max_row * Self::line_height(jvm, &this).await? + 2 * Self::PADDING)
    }

    async fn get_max_row(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxRow", "I").await
    }

    async fn set_max_row(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_row: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::setMaxRow({this:?}, {max_row})");

        if max_row <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxRow must be positive").await);
        }

        jvm.put_field(&mut this, "maxRow", "I", max_row).await
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Font>> {
        jvm.get_field(&this, "font", "Ljavax/microedition/lcdui/Font;").await
    }

    async fn set_font(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        if font.is_null() {
            return Ok(());
        }

        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", font).await
    }

    async fn next_input_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::nextInputMode({this:?})");

        if !jvm.get_field::<bool>(&this, "focus", "Z").await? {
            return Ok(Self::IM_NONE);
        }
        let constraints: i32 = jvm.get_field(&this, "constraints", "I").await?;
        let input_mode: i32 = jvm.get_field(&this, "inputMode", "I").await?;
        let next = match input_mode {
            _ if Self::numeric_only(constraints) => Self::IM_NUMERIC,
            Self::IM_ROMAN_SMALL => Self::IM_ROMAN_CAPS,
            Self::IM_ROMAN_CAPS => Self::IM_NUMERIC,
            _ => Self::IM_ROMAN_SMALL,
        };
        jvm.put_field(&mut this, "inputMode", "I", next).await?;
        jvm.put_field(&mut this, "tapKey", "I", Self::NO_KEY).await?;

        Ok(next)
    }

    async fn get_input_mode(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        if !jvm.get_field::<bool>(&this, "focus", "Z").await? {
            return Ok(Self::IM_NONE);
        }

        jvm.get_field(&this, "inputMode", "I").await
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn set_string(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::setString({this:?}, {text:?})");

        let text = if text.is_null() {
            RustString::new()
        } else {
            JavaLangString::to_rust_string(jvm, &text).await?
        };

        Self::replace_text(jvm, &mut this, &text).await
    }

    async fn get_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut data: ClassInstanceRef<Array<JavaChar>>,
    ) -> JvmResult<i32> {
        let chars = Self::text(jvm, &this).await?.encode_utf16().collect::<Vec<_>>();
        if chars.len() > jvm.array_length(&data).await? {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "data is too short").await);
        }
        let count = chars.len() as i32;
        jvm.store_array(&mut data, 0, chars).await?;

        Ok(count)
    }

    async fn chars_to_string(jvm: &Jvm, data: &ClassInstanceRef<Array<JavaChar>>, offset: i32, length: i32) -> JvmResult<RustString> {
        if data.is_null() {
            return Ok(RustString::new());
        }
        if offset < 0 || length < 0 || (offset + length) as usize > jvm.array_length(data).await? {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "range outside data").await);
        }
        let chars: Vec<JavaChar> = jvm.load_array(data, offset as _, length as _).await?;

        Ok(RustString::from_utf16_lossy(&chars))
    }

    async fn set_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        let text = Self::chars_to_string(jvm, &data, offset, length).await?;

        Self::replace_text(jvm, &mut this, &text).await
    }

    async fn insert_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, inserted: &str, position: i32) -> JvmResult<()> {
        let chars = Self::text(jvm, this).await?.chars().collect::<Vec<_>>();
        let position = (position.max(0) as usize).min(chars.len());
        let text = chars[..position]
            .iter()
            .copied()
            .chain(inserted.chars())
            .chain(chars[position..].iter().copied())
            .collect::<RustString>();

        Self::replace_text(jvm, this, &text).await
    }

    async fn insert(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        src: ClassInstanceRef<String>,
        position: i32,
    ) -> JvmResult<()> {
        let inserted = JavaLangString::to_rust_string(jvm, &src).await?;

        Self::insert_text(jvm, &mut this, &inserted, position).await
    }

    async fn insert_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        position: i32,
    ) -> JvmResult<()> {
        let inserted = Self::chars_to_string(jvm, &data, offset, length).await?;

        Self::insert_text(jvm, &mut this, &inserted, position).await
    }

    async fn delete(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, offset: i32, length: i32) -> JvmResult<()> {
        let chars = Self::text(jvm, &this).await?.chars().collect::<Vec<_>>();
        if offset < 0 || length < 0 || (offset + length) as usize > chars.len() {
            return Err(jvm
                .exception("java/lang/StringIndexOutOfBoundsException", "range outside the contents")
                .await);
        }
        let text = chars[..offset as usize]
            .iter()
            .chain(chars[(offset + length) as usize..].iter())
            .collect::<RustString>();

        Self::replace_text(jvm, &mut this, &text).await
    }

    async fn get_max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxSize", "I").await
    }

    async fn set_max_size(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<i32> {
        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must be positive").await);
        }
        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;

        let text = Self::text(jvm, &this).await?.chars().take(max_size as usize).collect::<RustString>();
        Self::replace_text(jvm, &mut this, &text).await?;

        Ok(max_size)
    }

    async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(Self::text(jvm, &this).await?.encode_utf16().count() as i32)
    }

    async fn get_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "constraints", "I").await
    }

    async fn set_constraints(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraints: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "constraints", "I", constraints).await?;
        jvm.put_field(&mut this, "inputMode", "I", Self::first_input_mode(constraints)).await
    }

    async fn set_focus(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::setFocus({this:?}, {focus})");

        jvm.put_field(&mut this, "focus", "Z", focus).await?;
        jvm.put_field(&mut this, "tapKey", "I", Self::NO_KEY).await
    }

    async fn has_focus(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "focus", "Z").await
    }

    /// The characters a keypad key cycles through in the roman modes, lower case.
    fn letters(key: u8) -> &'static str {
        match key {
            b'1' => ".,?!-@1",
            b'2' => "abc2",
            b'3' => "def3",
            b'4' => "ghi4",
            b'5' => "jkl5",
            b'6' => "mno6",
            b'7' => "pqrs7",
            b'8' => "tuv8",
            b'9' => "wxyz9",
            _ => " 0",
        }
    }

    async fn key_pressed(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::keyPressed({this:?}, {key_code})");

        if !jvm.get_field::<bool>(&this, "focus", "Z").await? {
            return Ok(());
        }
        // the title passes on what its Canvas received; name the key in the MIDP layer's own codes
        let key = match KeyMap::load(jvm).await? {
            Some(key_map) => key_map.internal_code(key_code),
            None => key_code,
        };
        let mut chars = Self::text(jvm, &this).await?.chars().collect::<Vec<_>>();
        let tap_key: i32 = jvm.get_field(&this, "tapKey", "I").await?;

        if key == MIDPKeyCode::CLEAR as i32 {
            chars.pop();
            jvm.put_field(&mut this, "tapKey", "I", Self::NO_KEY).await?;
        } else if (b'0' as i32..=b'9' as i32).contains(&key) {
            let input_mode: i32 = jvm.get_field(&this, "inputMode", "I").await?;
            let max_size: i32 = jvm.get_field(&this, "maxSize", "I").await?;
            let upper_case = input_mode == Self::IM_ROMAN_CAPS;

            if input_mode == Self::IM_NUMERIC {
                if chars.len() < max_size as usize {
                    chars.push(key as u8 as char);
                }
            } else {
                let letters = Self::letters(key as u8).chars().collect::<Vec<_>>();
                let cycling = tap_key == key && !chars.is_empty();
                let tap_index = if cycling {
                    (jvm.get_field::<i32>(&this, "tapIndex", "I").await? + 1) % letters.len() as i32
                } else {
                    0
                };
                let letter = if upper_case {
                    letters[tap_index as usize].to_ascii_uppercase()
                } else {
                    letters[tap_index as usize]
                };

                if cycling {
                    chars.pop();
                }
                if chars.len() < max_size as usize {
                    chars.push(letter);
                    jvm.put_field(&mut this, "tapKey", "I", key).await?;
                    jvm.put_field(&mut this, "tapIndex", "I", tap_index).await?;
                }
            }
        } else {
            // any other key ends the letter being cycled
            jvm.put_field(&mut this, "tapKey", "I", Self::NO_KEY).await?;
            return Ok(());
        }

        Self::store_text(jvm, &mut this, &chars.into_iter().collect::<RustString>()).await
    }

    async fn key_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::keyReleased({this:?}, {key_code})");

        Ok(())
    }

    /// Draws the field with its top-left corner at the origin of `graphics`.
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.TextFieldX::paint({this:?}, {graphics:?})");

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let max_row: i32 = jvm.get_field(&this, "maxRow", "I").await?;
        let constraints: i32 = jvm.get_field(&this, "constraints", "I").await?;
        let focus: bool = jvm.get_field(&this, "focus", "Z").await?;
        let font: ClassInstanceRef<Font> = jvm.get_field(&this, "font", "Ljavax/microedition/lcdui/Font;").await?;
        let line_height = Self::line_height(jvm, &this).await?;
        let height = max_row * line_height + 2 * Self::PADDING;

        let text = Self::text(jvm, &this).await?;
        let shown = if constraints & Self::PASSWORD != 0 {
            text.chars().map(|_| '*').collect::<RustString>()
        } else {
            text
        };

        // break into rows that fit the inner width and keep the last `maxRow` of them
        let inner_width = width - 2 * Self::PADDING;
        let mut rows: Vec<RustString> = vec![RustString::new()];
        for x in shown.chars() {
            let mut candidate = rows.last().unwrap().clone();
            candidate.push(x);
            let candidate_java = JavaLangString::from_rust_string(jvm, &candidate).await?;
            let candidate_width: i32 = jvm
                .invoke_virtual(&font, FONT, "stringWidth", "(Ljava/lang/String;)I", (candidate_java,))
                .await?;
            if candidate_width > inner_width && !rows.last().unwrap().is_empty() {
                rows.push(RustString::from(x));
            } else {
                *rows.last_mut().unwrap() = candidate;
            }
        }
        let first_row = rows.len().saturating_sub(max_row as usize);

        const TOP_LEFT: i32 = 16 | 4;
        let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "setColor", "(I)V", (0xffffff,)).await?;
        let _: () = jvm
            .invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (0, 0, width, height))
            .await?;
        let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "setColor", "(I)V", (0x000000,)).await?;
        let _: () = jvm
            .invoke_virtual(&graphics, GRAPHICS, "drawRect", "(IIII)V", (0, 0, width - 1, height - 1))
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, GRAPHICS, "setFont", "(Ljavax/microedition/lcdui/Font;)V", (font.clone(),))
            .await?;

        let mut caret_x = Self::PADDING;
        let mut caret_y = Self::PADDING;
        for (index, row) in rows[first_row..].iter().enumerate() {
            let y = Self::PADDING + index as i32 * line_height;
            let row_java = JavaLangString::from_rust_string(jvm, row).await?;
            let row_width: i32 = jvm
                .invoke_virtual(&font, FONT, "stringWidth", "(Ljava/lang/String;)I", (row_java.clone(),))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    GRAPHICS,
                    "drawString",
                    "(Ljava/lang/String;III)V",
                    (row_java, Self::PADDING, y, TOP_LEFT),
                )
                .await?;
            caret_x = Self::PADDING + row_width;
            caret_y = y;
        }
        if focus {
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    GRAPHICS,
                    "drawLine",
                    "(IIII)V",
                    (caret_x, caret_y, caret_x, caret_y + line_height - 1),
                )
                .await?;
        }

        Ok(())
    }
}
