use alloc::{string::String as RustString, vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::text_layout::preferred_width;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Graphics, Item};
use crate::classes::net::wie::MIDPKeyCode;

const BOX_INSET: i32 = 3;
const MINIMUM_BOX_WIDTH: i32 = 40;
const BOX_BACKGROUND: i32 = 0xffffff;
const BOX_BORDER: i32 = 0x65727d;
const FOCUS_BORDER: i32 = 0x2f6f9f;
const LEFT_TOP: i32 = 4 | 16;

// class javax.microedition.lcdui.TextField
//
// A single-line editable Item. The keypad types digits into it and CLR deletes; the SK-VM titles
// that reference the class only read its constraint constants, which javac did not inline for them.
pub struct TextField;

impl TextField {
    pub const ANY: i32 = 0;
    pub const EMAILADDR: i32 = 1;
    pub const NUMERIC: i32 = 2;
    pub const PHONENUMBER: i32 = 3;
    pub const URL: i32 = 4;
    pub const DECIMAL: i32 = 5;
    pub const PASSWORD: i32 = 0x10000;
    pub const UNEDITABLE: i32 = 0x20000;
    pub const SENSITIVE: i32 = 0x40000;
    pub const NON_PREDICTIVE: i32 = 0x80000;
    pub const INITIAL_CAPS_WORD: i32 = 0x100000;
    pub const INITIAL_CAPS_SENTENCE: i32 = 0x200000;
    pub const CONSTRAINT_MASK: i32 = 0xffff;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/TextField",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
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
                JavaMethodProto::new("getCaretPosition", "()I", Self::get_caret_position, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getConstraints", "()I", Self::get_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setConstraints", "(I)V", Self::set_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setInitialInputMode",
                    "(Ljava/lang/String;)V",
                    Self::set_initial_input_mode,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("minimumContentWidth", "()I", Self::minimum_content_width, MethodAccessFlags::empty()),
                JavaMethodProto::new("minimumContentHeight", "()I", Self::minimum_content_height, MethodAccessFlags::empty()),
                JavaMethodProto::new("preferredContentWidth", "()I", Self::preferred_content_width, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "preferredContentHeight",
                    "(I)I",
                    Self::preferred_content_height,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "paintContent",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    Self::paint_content,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("isFocusable", "()Z", Self::is_focusable, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleItemKey", "(I)I", Self::handle_item_key, MethodAccessFlags::empty()),
            ],
            fields: vec![
                JavaFieldProto::new("ANY", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL),
                JavaFieldProto::new(
                    "EMAILADDR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "NUMERIC",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "PHONENUMBER",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("URL", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL),
                JavaFieldProto::new(
                    "DECIMAL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "PASSWORD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "UNEDITABLE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SENSITIVE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "NON_PREDICTIVE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INITIAL_CAPS_WORD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INITIAL_CAPS_SENTENCE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "CONSTRAINT_MASK",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("constraints", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::<clinit>");

        for (field, value) in [
            ("ANY", Self::ANY),
            ("EMAILADDR", Self::EMAILADDR),
            ("NUMERIC", Self::NUMERIC),
            ("PHONENUMBER", Self::PHONENUMBER),
            ("URL", Self::URL),
            ("DECIMAL", Self::DECIMAL),
            ("PASSWORD", Self::PASSWORD),
            ("UNEDITABLE", Self::UNEDITABLE),
            ("SENSITIVE", Self::SENSITIVE),
            ("NON_PREDICTIVE", Self::NON_PREDICTIVE),
            ("INITIAL_CAPS_WORD", Self::INITIAL_CAPS_WORD),
            ("INITIAL_CAPS_SENTENCE", Self::INITIAL_CAPS_SENTENCE),
            ("CONSTRAINT_MASK", Self::CONSTRAINT_MASK),
        ] {
            jvm.put_static_field("javax/microedition/lcdui/TextField", field, "I", value).await?;
        }

        Ok(())
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::<init>({this:?}, {label:?}, {text:?}, {max_size}, {constraints})");

        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must be positive").await);
        }
        Self::check_constraints(jvm, constraints).await?;

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Item", "setLabel", "(Ljava/lang/String;)V", (label,))
            .await?;

        let text = if text.is_null() {
            Vec::new()
        } else {
            JavaLangString::to_rust_string(jvm, &text).await?.encode_utf16().collect()
        };
        if text.len() as i32 > max_size {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "text exceeds maxSize").await);
        }

        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;
        jvm.put_field(&mut this, "constraints", "I", constraints).await?;
        Self::store_text(jvm, &mut this, &text).await
    }

    async fn check_constraints(jvm: &Jvm, constraints: i32) -> JvmResult<()> {
        if !(Self::ANY..=Self::DECIMAL).contains(&(constraints & Self::CONSTRAINT_MASK)) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid constraints").await);
        }
        Ok(())
    }

    async fn get_string(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.TextField::getString({this:?})");

        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn set_string(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setString({this:?}, {text:?})");

        let text = if text.is_null() {
            Vec::new()
        } else {
            JavaLangString::to_rust_string(jvm, &text).await?.encode_utf16().collect()
        };
        Self::replace_text(jvm, &mut this, text).await
    }

    async fn get_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getChars({this:?}, {data:?})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }
        let text = Self::text(jvm, &this).await?;
        if jvm.array_length(&data).await? < text.len() {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "data is too short").await);
        }

        let mut data = data;
        jvm.store_array(&mut data, 0, text.iter().copied()).await?;

        Ok(text.len() as i32)
    }

    async fn set_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setChars({this:?}, {data:?}, {offset}, {length})");

        let text = if data.is_null() {
            Vec::new()
        } else {
            Self::chars_of(jvm, &data, offset, length).await?
        };
        Self::replace_text(jvm, &mut this, text).await
    }

    async fn insert(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        source: ClassInstanceRef<String>,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::insert({this:?}, {source:?}, {position})");

        if source.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "src is null").await);
        }
        let source: Vec<u16> = JavaLangString::to_rust_string(jvm, &source).await?.encode_utf16().collect();
        Self::insert_at(jvm, &mut this, &source, position).await
    }

    async fn insert_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::insert({this:?}, {data:?}, {offset}, {length}, {position})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }
        let source = Self::chars_of(jvm, &data, offset, length).await?;
        Self::insert_at(jvm, &mut this, &source, position).await
    }

    async fn delete(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, offset: i32, length: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::delete({this:?}, {offset}, {length})");

        let mut text = Self::text(jvm, &this).await?;
        if offset < 0 || length < 0 || offset.checked_add(length).is_none_or(|end| end as usize > text.len()) {
            return Err(jvm
                .exception("java/lang/StringIndexOutOfBoundsException", "delete range is out of bounds")
                .await);
        }
        text.drain(offset as usize..(offset + length) as usize);
        Self::store_text(jvm, &mut this, &text).await?;
        Self::changed(jvm, &this).await
    }

    async fn get_max_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getMaxSize({this:?})");

        jvm.get_field(&this, "maxSize", "I").await
    }

    async fn set_max_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::setMaxSize({this:?}, {max_size})");

        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must be positive").await);
        }

        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;
        let mut text = Self::text(jvm, &this).await?;
        if text.len() as i32 > max_size {
            text.truncate(max_size as usize);
            Self::store_text(jvm, &mut this, &text).await?;
            Self::changed(jvm, &this).await?;
        }

        Ok(max_size)
    }

    async fn size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::size({this:?})");

        Ok(Self::text(jvm, &this).await?.len() as i32)
    }

    // The caret sits at the end: the keypad only appends and deletes there.
    async fn get_caret_position(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getCaretPosition({this:?})");

        Ok(Self::text(jvm, &this).await?.len() as i32)
    }

    async fn get_constraints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getConstraints({this:?})");

        jvm.get_field(&this, "constraints", "I").await
    }

    async fn set_constraints(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraints: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setConstraints({this:?}, {constraints})");

        Self::check_constraints(jvm, constraints).await?;
        jvm.put_field(&mut this, "constraints", "I", constraints).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
            .await
    }

    async fn set_initial_input_mode(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mode: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setInitialInputMode({this:?}, {mode:?})");

        Ok(())
    }

    async fn minimum_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::minimumContentWidth({this:?})");

        Ok(MINIMUM_BOX_WIDTH)
    }

    async fn minimum_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::minimumContentHeight({this:?})");

        Ok(Font::HEIGHT + BOX_INSET * 2)
    }

    async fn preferred_content_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::preferredContentWidth({this:?})");

        let shown = Self::shown_text(jvm, &this).await?;
        let text_width = preferred_width(context.system().platform().font(), &shown, 10.0);

        Ok((text_width + BOX_INSET * 2).max(MINIMUM_BOX_WIDTH))
    }

    async fn preferred_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::preferredContentHeight({this:?}, {width})");

        Ok(Font::HEIGHT + BOX_INSET * 2)
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_content(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        focused: bool,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::paintContent({this:?}, {graphics:?}, {x}, {y}, {width}, {height}, {focused})");

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (BOX_BACKGROUND,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (x, y, width, height),
            )
            .await?;
        let border = if focused { FOCUS_BORDER } else { BOX_BORDER };
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (border,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawRect",
                "(IIII)V",
                (x, y, width - 1, height - 1),
            )
            .await?;

        let shown = Self::shown_text(jvm, &this).await?;
        if shown.is_empty() {
            return Ok(());
        }

        let clip_x: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
            .await?;
        let clip_y: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
            .await?;
        let clip_width: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
            .await?;
        let clip_height: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "clipRect",
                "(IIII)V",
                (
                    x + BOX_INSET,
                    y + BOX_INSET,
                    (width - BOX_INSET * 2).max(0),
                    (height - BOX_INSET * 2).max(0),
                ),
            )
            .await?;

        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setFont",
                "(Ljavax/microedition/lcdui/Font;)V",
                (font,),
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (Item::TEXT_COLOR,))
            .await?;
        let java_text = JavaLangString::from_rust_string(jvm, &shown).await?;
        let draw_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawString",
                "(Ljava/lang/String;III)V",
                (java_text, x + BOX_INSET, y + BOX_INSET, LEFT_TOP),
            )
            .await;
        let restore_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setClip",
                "(IIII)V",
                (clip_x, clip_y, clip_width, clip_height),
            )
            .await;
        draw_result?;
        restore_result
    }

    async fn is_focusable(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.TextField::isFocusable({this:?})");

        Ok(true)
    }

    // Digits append, CLR deletes the last character; everything else is the Form's.
    async fn handle_item_key(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::handleItemKey({this:?}, {key})");

        let constraints: i32 = jvm.get_field(&this, "constraints", "I").await?;
        if constraints & Self::UNEDITABLE != 0 {
            return Ok(0);
        }

        let mut text = Self::text(jvm, &this).await?;
        if key == MIDPKeyCode::CLEAR as i32 {
            if text.pop().is_none() {
                return Ok(Item::INPUT_HANDLED);
            }
        } else if (MIDPKeyCode::KEY_NUM0 as i32..=MIDPKeyCode::KEY_NUM9 as i32).contains(&key) {
            let max_size: i32 = jvm.get_field(&this, "maxSize", "I").await?;
            if text.len() as i32 >= max_size {
                return Ok(Item::INPUT_HANDLED);
            }
            text.push(key as u16);
        } else {
            return Ok(0);
        }

        Self::store_text(jvm, &mut this, &text).await?;
        let _: () = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await?;

        Ok(Item::INPUT_HANDLED | Item::INPUT_CHANGED)
    }

    async fn text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<u16>> {
        let text: ClassInstanceRef<String> = jvm.get_field(this, "text", "Ljava/lang/String;").await?;
        if text.is_null() {
            return Ok(Vec::new());
        }

        Ok(JavaLangString::to_rust_string(jvm, &text).await?.encode_utf16().collect())
    }

    // What is painted: the text, or one asterisk per character of a PASSWORD field.
    async fn shown_text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<RustString> {
        let text = Self::text(jvm, this).await?;
        let constraints: i32 = jvm.get_field(this, "constraints", "I").await?;
        if constraints & Self::PASSWORD != 0 {
            return Ok(core::iter::repeat_n('*', text.len()).collect());
        }

        Ok(RustString::from_utf16_lossy(&text))
    }

    async fn store_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: &[u16]) -> JvmResult<()> {
        let text = JavaLangString::from_rust_string(jvm, &RustString::from_utf16_lossy(text)).await?;
        jvm.put_field(this, "text", "Ljava/lang/String;", text).await
    }

    async fn replace_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: Vec<u16>) -> JvmResult<()> {
        let max_size: i32 = jvm.get_field(this, "maxSize", "I").await?;
        if text.len() as i32 > max_size {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "text exceeds maxSize").await);
        }
        Self::store_text(jvm, this, &text).await?;
        Self::changed(jvm, this).await
    }

    async fn insert_at(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, source: &[u16], position: i32) -> JvmResult<()> {
        let mut text = Self::text(jvm, this).await?;
        let max_size: i32 = jvm.get_field(this, "maxSize", "I").await?;
        if (text.len() + source.len()) as i32 > max_size {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "text would exceed maxSize").await);
        }
        let position = position.clamp(0, text.len() as i32) as usize;
        text.splice(position..position, source.iter().copied());
        Self::store_text(jvm, this, &text).await?;
        Self::changed(jvm, this).await
    }

    async fn chars_of(jvm: &Jvm, data: &ClassInstanceRef<Array<JavaChar>>, offset: i32, length: i32) -> JvmResult<Vec<u16>> {
        let available = jvm.array_length(data).await? as i32;
        if offset < 0 || length < 0 || offset.checked_add(length).is_none_or(|end| end > available) {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "range is out of bounds").await);
        }

        jvm.load_array(data, offset as _, length as _).await
    }

    async fn changed(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.invoke_virtual(this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;

    use crate::{
        classes::{
            javax::microedition::lcdui::{Item, TextField},
            net::wie::MIDPKeyCode,
        },
        get_protos,
    };

    #[test]
    fn constants_text_and_keypad_editing() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let password: i32 = jvm.get_static_field("javax/microedition/lcdui/TextField", "PASSWORD", "I").await?;
            assert_eq!(password, 0x10000);
            let mask: i32 = jvm.get_static_field("javax/microedition/lcdui/TextField", "CONSTRAINT_MASK", "I").await?;
            assert_eq!(mask, 0xffff);

            let label = JavaLangString::from_rust_string(&jvm, "Name").await?;
            let text = JavaLangString::from_rust_string(&jvm, "12").await?;
            let field: ClassInstanceRef<TextField> = jvm
                .new_class(
                    "javax/microedition/lcdui/TextField",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    (label, text, 3, TextField::NUMERIC),
                )
                .await?
                .into();

            let size: i32 = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "size", "()I", ())
                .await?;
            assert_eq!(size, 2);

            let result: i32 = jvm
                .invoke_virtual(
                    &field,
                    "javax/microedition/lcdui/TextField",
                    "handleItemKey",
                    "(I)I",
                    (MIDPKeyCode::KEY_NUM7 as i32,),
                )
                .await?;
            assert_eq!(result, Item::INPUT_HANDLED | Item::INPUT_CHANGED);
            let result: i32 = jvm
                .invoke_virtual(
                    &field,
                    "javax/microedition/lcdui/TextField",
                    "handleItemKey",
                    "(I)I",
                    (MIDPKeyCode::KEY_NUM8 as i32,),
                )
                .await?;
            assert_eq!(result, Item::INPUT_HANDLED, "a full field takes the key and keeps the text");
            let string: ClassInstanceRef<String> = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getString", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &string).await?, "127");

            let result: i32 = jvm
                .invoke_virtual(
                    &field,
                    "javax/microedition/lcdui/TextField",
                    "handleItemKey",
                    "(I)I",
                    (MIDPKeyCode::CLEAR as i32,),
                )
                .await?;
            assert_eq!(result, Item::INPUT_HANDLED | Item::INPUT_CHANGED);
            let size: i32 = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "size", "()I", ())
                .await?;
            assert_eq!(size, 2);

            let long = JavaLangString::from_rust_string(&jvm, "1234").await?;
            let too_long: JvmResult<()> = jvm
                .invoke_virtual(
                    &field,
                    "javax/microedition/lcdui/TextField",
                    "setString",
                    "(Ljava/lang/String;)V",
                    (long,),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = too_long else {
                panic!("setString accepted text over maxSize");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let _: () = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "delete", "(II)V", (0, 1))
                .await?;
            let string: ClassInstanceRef<String> = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getString", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &string).await?, "2");

            Ok(())
        });

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
