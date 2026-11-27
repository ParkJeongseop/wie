use alloc::{string::String, vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{
    lang::{Object, String as JavaString},
    util::Hashtable,
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    net::wie::{ComponentState, WIPIKeyCode},
    org::kwis::msp::{
        lcdui::{Display, Font, Graphics},
        lwc::{Component, ShellComponent},
    },
};

const KEY_PRESSED: i32 = 1;

const ANCHOR_TOP_LEFT: i32 = 16 | 4;
const ANCHOR_TOP_HCENTER: i32 = 16 | 1;

const COLOR_BACKGROUND: i32 = 0xffffff;
const COLOR_FOREGROUND: i32 = 0x000000;
const COLOR_FRAME: i32 = 0x808080;
const COLOR_BUTTON: i32 = 0xd0d0d0;
const COLOR_HIGHLIGHT: i32 = 0x3060c0;

const DRAWN_CLASSES: [&str; 4] = [
    "org/kwis/msp/lwc/LabelComponent",
    "org/kwis/msp/lwc/TextComponent",
    "org/kwis/msp/lwc/ButtonComponent",
    "com/ktf/kfc/ChoiceText",
];

const MARGIN: i32 = 4;
const PADDING: i32 = 3;

// class net.wie.ShellCard: the lcdui card a shown lwc shell sits on. A shell is linked to a card
// and takes the card's events; showing a shell pushes one of these, which has the shell draw itself
// and hands it the keys. A shell the title neither draws nor takes the keys of is the SDK's own:
// its title string, labels, text components, choices and buttons are drawn one to a row, UP and
// DOWN move the focus, digits and CLR edit the focused text, LEFT and RIGHT change the focused
// choice, and OK presses the focused button. Over a shell the title draws, only the components it
// placed itself are drawn, where it placed them. The card of each shown shell is kept in a static
// table rather than on the shell, because titles subclass shells and lay their own fields out
// after the SDK's.
pub struct ShellCard;

impl ShellCard {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/ShellCard",
            parent_class: Some("org/kwis/msp/lcdui/Card"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lwc/ShellComponent;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![
                JavaFieldProto::new("shell", "Lorg/kwis/msp/lwc/ShellComponent;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cards", "Ljava/util/Hashtable;", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, shell: ClassInstanceRef<ShellComponent>) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::<init>({this:?}, {shell:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "net/wie/ShellCard", "shell", "Lorg/kwis/msp/lwc/ShellComponent;", shell)
            .await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::paint({this:?}, {graphics:?})");

        let shell: ClassInstanceRef<ShellComponent> = jvm
            .get_field(&this, "net/wie/ShellCard", "shell", "Lorg/kwis/msp/lwc/ShellComponent;")
            .await?;
        // A shell subclass draws itself by overriding `paint` (LGT 학교가는길) or the specification's
        // `paintContent`.
        for name in ["paint", "paintContent"] {
            if let Some(class_name) = Self::declaring_class(jvm, &shell, name, "(Lorg/kwis/msp/lcdui/Graphics;)V") {
                let _: () = jvm
                    .invoke_virtual(&shell, &class_name, name, "(Lorg/kwis/msp/lcdui/Graphics;)V", (graphics.clone(),))
                    .await?;

                return Self::paint_components(jvm, &this, &shell, &graphics, true).await;
            }
        }

        Self::paint_components(jvm, &this, &shell, &graphics, false).await
    }

    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("net.wie.ShellCard::keyNotify({this:?}, {type}, {key})");

        let shell: ClassInstanceRef<ShellComponent> = jvm
            .get_field(&this, "net/wie/ShellCard", "shell", "Lorg/kwis/msp/lwc/ShellComponent;")
            .await?;

        if Self::declaring_class(jvm, &shell, "keyNotify", "(II)Z").as_deref() != Some("org/kwis/msp/lwc/Component") {
            return jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "keyNotify", "(II)Z", (r#type, key))
                .await;
        }

        if r#type == KEY_PRESSED {
            Self::key_pressed(jvm, &shell, key).await?;
            let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lcdui/Card", "repaint", "()V", ()).await?;
        }

        Ok(true)
    }

    async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, shown: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::showNotify({this:?}, {shown})");

        let shell: ClassInstanceRef<ShellComponent> = jvm
            .get_field(&this, "net/wie/ShellCard", "shell", "Lorg/kwis/msp/lwc/ShellComponent;")
            .await?;

        jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "showNotify", "(Z)V", (shown,))
            .await
    }

    /// The labels, text components, choices and buttons `shell` holds, in the order they were
    /// added; containers are walked into.
    async fn components(jvm: &Jvm, shell: &ClassInstanceRef<ShellComponent>) -> JvmResult<Vec<ClassInstanceRef<Component>>> {
        let mut components = Vec::new();
        let mut pending = Self::children(jvm, shell).await?;
        while let Some(component) = pending.pop() {
            if jvm.is_instance(&**component, "org/kwis/msp/lwc/ContainerComponent") {
                pending.extend(Self::children(jvm, &component).await?);
            } else if DRAWN_CLASSES.iter().any(|x| jvm.is_instance(&**component, x)) {
                components.push(component);
            }
        }

        Ok(components)
    }

    /// The children of `container`, last first.
    async fn children<T: Send + Sync>(jvm: &Jvm, container: &ClassInstanceRef<T>) -> JvmResult<Vec<ClassInstanceRef<Component>>> {
        let count: i32 = jvm
            .invoke_virtual(container, "org/kwis/msp/lwc/ContainerComponent", "getComponentCount", "()I", ())
            .await?;

        let mut children = Vec::new();
        for index in (0..count).rev() {
            let child: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    container,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    (index,),
                )
                .await?;
            if !child.is_null() {
                children.push(child);
            }
        }

        Ok(children)
    }

    /// Which of `components` takes the keys: the one that has the focus, or else the first that
    /// can have it. Labels cannot.
    async fn focus(jvm: &Jvm, components: &[ClassInstanceRef<Component>]) -> JvmResult<Option<usize>> {
        let mut first = None;
        for (index, component) in components.iter().enumerate() {
            if jvm.is_instance(&***component, "org/kwis/msp/lwc/LabelComponent") {
                continue;
            }
            if jvm.invoke_virtual(component, "org/kwis/msp/lwc/Component", "hasFocus", "()Z", ()).await? {
                return Ok(Some(index));
            }
            first = first.or(Some(index));
        }

        Ok(first)
    }

    /// Draws the components of `shell`. Over a shell the title has `drawn`, those are the ones it
    /// placed; otherwise the shell gets the SDK's own look, the title string in a bar and then each
    /// component that was not placed on a row of its own.
    async fn paint_components(
        jvm: &Jvm,
        this: &ClassInstanceRef<Self>,
        shell: &ClassInstanceRef<ShellComponent>,
        graphics: &ClassInstanceRef<Graphics>,
        drawn: bool,
    ) -> JvmResult<()> {
        let width: i32 = jvm.invoke_virtual(this, "org/kwis/msp/lcdui/Card", "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_virtual(this, "org/kwis/msp/lcdui/Card", "getHeight", "()I", ()).await?;
        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("org/kwis/msp/lcdui/Font", "getDefaultFont", "()Lorg/kwis/msp/lcdui/Font;", ())
            .await?;
        let font_height: i32 = jvm.invoke_virtual(&font, "org/kwis/msp/lcdui/Font", "getHeight", "()I", ()).await?;
        let row = font_height + 2 * PADDING;

        let mut y = 0;
        if !drawn {
            Self::fill_rect(jvm, graphics, COLOR_BACKGROUND, 0, 0, width, height).await?;

            let title = ComponentState::text(jvm, shell).await?;
            if !title.is_null() {
                Self::fill_rect(jvm, graphics, COLOR_HIGHLIGHT, 0, 0, width, row).await?;
                Self::draw_string(jvm, graphics, COLOR_BACKGROUND, title, MARGIN, PADDING, ANCHOR_TOP_LEFT).await?;
                y = row;
            }
        }

        let components = Self::components(jvm, shell).await?;
        let focus = Self::focus(jvm, &components).await?;
        for (index, component) in components.iter().enumerate() {
            let bounds = match ComponentState::bounds(jvm, component).await? {
                Some(bounds) => bounds,
                None if drawn => continue,
                None => {
                    y += MARGIN + row;
                    [MARGIN, y - row, width - 2 * MARGIN, row]
                }
            };

            Self::paint_component(jvm, graphics, component, focus == Some(index), bounds, font_height).await?;
        }

        Ok(())
    }

    /// Draws one component in the rectangle `bounds` (x, y, width, height).
    async fn paint_component(
        jvm: &Jvm,
        graphics: &ClassInstanceRef<Graphics>,
        component: &ClassInstanceRef<Component>,
        focused: bool,
        [x, y, width, height]: [i32; 4],
        font_height: i32,
    ) -> JvmResult<()> {
        let frame = if focused { COLOR_HIGHLIGHT } else { COLOR_FRAME };
        let text_y = y + (height - font_height) / 2;

        if jvm.is_instance(&***component, "org/kwis/msp/lwc/TextComponent") {
            let text: ClassInstanceRef<JavaString> = jvm
                .invoke_virtual(component, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
                .await?;
            Self::fill_rect(jvm, graphics, frame, x, y, width, height).await?;
            Self::fill_rect(jvm, graphics, COLOR_BACKGROUND, x + 1, y + 1, width - 2, height - 2).await?;
            Self::draw_string(jvm, graphics, COLOR_FOREGROUND, text, x + PADDING, text_y, ANCHOR_TOP_LEFT).await
        } else if jvm.is_instance(&***component, "com/ktf/kfc/ChoiceText") {
            let (text, selected) = Self::choices(jvm, component).await?;
            let text = text.into_iter().nth(selected as usize).unwrap_or(None.into());
            Self::fill_rect(jvm, graphics, frame, x, y, width, height).await?;
            Self::fill_rect(jvm, graphics, COLOR_BACKGROUND, x + 1, y + 1, width - 2, height - 2).await?;
            Self::draw_string(jvm, graphics, COLOR_FOREGROUND, text, x + width / 2, text_y, ANCHOR_TOP_HCENTER).await
        } else if jvm.is_instance(&***component, "org/kwis/msp/lwc/ButtonComponent") {
            let image = ComponentState::image(jvm, component).await?;
            if !image.is_null() {
                if focused {
                    Self::fill_rect(jvm, graphics, COLOR_HIGHLIGHT, x - 1, y - 1, width + 2, height + 2).await?;
                }

                return jvm
                    .invoke_virtual(
                        graphics,
                        "org/kwis/msp/lcdui/Graphics",
                        "drawImage",
                        "(Lorg/kwis/msp/lcdui/Image;III)V",
                        (image, x, y, ANCHOR_TOP_LEFT),
                    )
                    .await;
            }

            let (fill, color) = if focused {
                (COLOR_HIGHLIGHT, COLOR_BACKGROUND)
            } else {
                (COLOR_BUTTON, COLOR_FOREGROUND)
            };
            let text = ComponentState::text(jvm, component).await?;
            Self::fill_rect(jvm, graphics, fill, x, y, width, height).await?;
            Self::draw_string(jvm, graphics, color, text, x + width / 2, text_y, ANCHOR_TOP_HCENTER).await
        } else {
            let text = ComponentState::text(jvm, component).await?;
            Self::draw_string(jvm, graphics, COLOR_FOREGROUND, text, x, text_y, ANCHOR_TOP_LEFT).await
        }
    }

    /// The strings of a `ChoiceText` and which of them is selected.
    async fn choices(jvm: &Jvm, component: &ClassInstanceRef<Component>) -> JvmResult<(Vec<ClassInstanceRef<JavaString>>, i32)> {
        let text: ClassInstanceRef<Array<ClassInstanceRef<JavaString>>> = jvm
            .invoke_virtual(component, "com/ktf/kfc/ChoiceText", "getAllText", "()[Ljava/lang/String;", ())
            .await?;
        let selected: i32 = jvm
            .invoke_virtual(component, "com/ktf/kfc/ChoiceText", "getSelectedIndex", "()I", ())
            .await?;
        if text.is_null() {
            return Ok((Vec::new(), selected));
        }
        let length = jvm.array_length(&text).await?;

        Ok((jvm.load_array(&text, 0, length).await?, selected))
    }

    async fn fill_rect(jvm: &Jvm, graphics: &ClassInstanceRef<Graphics>, color: i32, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        let _: () = jvm
            .invoke_virtual(graphics, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;

        jvm.invoke_virtual(graphics, "org/kwis/msp/lcdui/Graphics", "fillRect", "(IIII)V", (x, y, width, height))
            .await
    }

    /// Draws `string` unless the component has none.
    async fn draw_string(
        jvm: &Jvm,
        graphics: &ClassInstanceRef<Graphics>,
        color: i32,
        string: ClassInstanceRef<JavaString>,
        x: i32,
        y: i32,
        anchor: i32,
    ) -> JvmResult<()> {
        if string.is_null() {
            return Ok(());
        }
        let _: () = jvm
            .invoke_virtual(graphics, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;

        jvm.invoke_virtual(
            graphics,
            "org/kwis/msp/lcdui/Graphics",
            "drawString",
            "(Ljava/lang/String;III)V",
            (string, x, y, anchor),
        )
        .await
    }

    /// The SDK's own handling of a key on a shell: UP and DOWN move the focus, digits and CLR edit
    /// the focused text, LEFT and RIGHT change the focused choice, OK on a button calls its listener
    /// and anywhere else moves on to the next component.
    async fn key_pressed(jvm: &Jvm, shell: &ClassInstanceRef<ShellComponent>, key: i32) -> JvmResult<()> {
        let components = Self::components(jvm, shell).await?;
        let Some(focus) = Self::focus(jvm, &components).await? else {
            return Ok(());
        };
        let component = &components[focus];
        let is_text = jvm.is_instance(&***component, "org/kwis/msp/lwc/TextComponent");
        let is_choice = jvm.is_instance(&***component, "com/ktf/kfc/ChoiceText");
        let is_button = jvm.is_instance(&***component, "org/kwis/msp/lwc/ButtonComponent");

        match WIPIKeyCode::from_raw(key) {
            Some(WIPIKeyCode::UP) => Self::move_focus(jvm, &components, focus, false).await,
            Some(WIPIKeyCode::DOWN) => Self::move_focus(jvm, &components, focus, true).await,
            Some(WIPIKeyCode::FIRE) if !is_button => Self::move_focus(jvm, &components, focus, true).await,
            Some(WIPIKeyCode::FIRE) => {
                let (listener, data) = ComponentState::listener(jvm, component).await?;
                if listener.is_null() {
                    return Ok(());
                }

                jvm.invoke_virtual(
                    &listener,
                    "org/kwis/msp/lwc/ActionListener",
                    "action",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/Object;)V",
                    (component.clone(), data),
                )
                .await
            }
            Some(WIPIKeyCode::LEFT) if is_choice => Self::choose(jvm, component, -1).await,
            Some(WIPIKeyCode::RIGHT) if is_choice => Self::choose(jvm, component, 1).await,
            Some(WIPIKeyCode::CLEAR) if is_text => Self::edit(jvm, component, None).await,
            Some(_) if is_text && (WIPIKeyCode::NUM0 as i32..=WIPIKeyCode::NUM9 as i32).contains(&key) => {
                Self::edit(jvm, component, char::from_u32(key as u32)).await
            }
            _ => Ok(()),
        }
    }

    /// Gives the focus to the component after (or before) the focused one that can have it.
    async fn move_focus(jvm: &Jvm, components: &[ClassInstanceRef<Component>], focus: usize, forward: bool) -> JvmResult<()> {
        let can_focus = |index: &usize| !jvm.is_instance(&**components[*index], "org/kwis/msp/lwc/LabelComponent");
        let next = if forward {
            (focus + 1..components.len()).find(can_focus)
        } else {
            (0..focus).rev().find(can_focus)
        };
        let Some(next) = next else {
            return Ok(());
        };

        let _: () = jvm
            .invoke_virtual(&components[focus], "org/kwis/msp/lwc/Component", "focusNotify", "(Z)V", (false,))
            .await?;

        jvm.invoke_virtual(&components[next], "org/kwis/msp/lwc/Component", "focusNotify", "(Z)V", (true,))
            .await
    }

    /// Selects the string `step` after the selected one of a `ChoiceText`, going round at the ends.
    async fn choose(jvm: &Jvm, component: &ClassInstanceRef<Component>, step: i32) -> JvmResult<()> {
        let (text, selected) = Self::choices(jvm, component).await?;
        if text.is_empty() {
            return Ok(());
        }
        let selected = (selected + step).rem_euclid(text.len() as i32);
        let _: bool = jvm
            .invoke_virtual(component, "com/ktf/kfc/ChoiceText", "setSelectedIndex", "(I)Z", (selected,))
            .await?;

        Ok(())
    }

    /// Appends `typed` to the text of `component` while its maximum length allows, or removes its
    /// last character when nothing was typed.
    async fn edit(jvm: &Jvm, component: &ClassInstanceRef<Component>, typed: Option<char>) -> JvmResult<()> {
        let text: ClassInstanceRef<JavaString> = jvm
            .invoke_virtual(component, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
            .await?;
        let mut text = JavaLangString::to_rust_string(jvm, &text).await?;

        if let Some(typed) = typed {
            let max_length: i32 = jvm.get_field(component, "org/kwis/msp/lwc/TextComponent", "maxLength", "I").await?;
            if max_length > 0 && text.chars().count() >= max_length as usize {
                return Ok(());
            }
            text.push(typed);
        } else {
            text.pop();
        }

        let text: ClassInstanceRef<JavaString> = JavaLangString::from_rust_string(jvm, &text).await?.into();

        jvm.invoke_virtual(component, "org/kwis/msp/lwc/TextComponent", "setString", "(Ljava/lang/String;)V", (text,))
            .await
    }

    /// The class in `instance`'s hierarchy that declares the method, if any does.
    fn declaring_class(jvm: &Jvm, instance: &ClassInstanceRef<ShellComponent>, name: &str, descriptor: &str) -> Option<String> {
        let mut class = jvm.get_class(&instance.class_definition().name());
        while let Some(current) = class {
            if current.definition.method(name, descriptor, false).is_some() {
                return Some(current.definition.name().into_owned());
            }
            class = current.definition.super_class_name().and_then(|x| jvm.get_class(&x));
        }

        None
    }

    async fn cards(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Hashtable>> {
        let mut table: ClassInstanceRef<Hashtable> = jvm.get_static_field("net/wie/ShellCard", "cards", "Ljava/util/Hashtable;").await?;
        if table.is_null() {
            table = jvm.new_class("java/util/Hashtable", "()V", ()).await?.into();
            jvm.put_static_field("net/wie/ShellCard", "cards", "Ljava/util/Hashtable;", table.clone())
                .await?;
        }

        Ok(table)
    }

    /// The card `component` is shown on, if it is a shown shell.
    pub async fn card_of<T: Send + Sync>(jvm: &Jvm, component: &ClassInstanceRef<T>) -> JvmResult<ClassInstanceRef<ShellCard>> {
        let table = Self::cards(jvm).await?;

        jvm.invoke_virtual(
            &table,
            "java/util/Hashtable",
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            (component.clone(),),
        )
        .await
    }

    /// Puts `shell` on screen on a card of its own; a shell that is already shown stays where it is.
    pub async fn show(jvm: &Jvm, shell: &ClassInstanceRef<ShellComponent>) -> JvmResult<()> {
        let table = Self::cards(jvm).await?;
        let card = Self::card_of(jvm, shell).await?;
        if !card.is_null() {
            return Ok(());
        }

        let card: ClassInstanceRef<ShellCard> = jvm
            .new_class("net/wie/ShellCard", "(Lorg/kwis/msp/lwc/ShellComponent;)V", (shell.clone(),))
            .await?
            .into();
        let _: ClassInstanceRef<Object> = jvm
            .invoke_virtual(
                &table,
                "java/util/Hashtable",
                "put",
                "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                (shell.clone(), card.clone()),
            )
            .await?;

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
            .await?;
        jvm.invoke_virtual(
            &display,
            "org/kwis/msp/lcdui/Display",
            "pushCard",
            "(Lorg/kwis/msp/lcdui/Card;)V",
            (card,),
        )
        .await
    }

    /// Takes `shell`'s card off the screen.
    pub async fn hide(jvm: &Jvm, shell: &ClassInstanceRef<ShellComponent>) -> JvmResult<()> {
        let table = Self::cards(jvm).await?;
        let card: ClassInstanceRef<ShellCard> = jvm
            .invoke_virtual(
                &table,
                "java/util/Hashtable",
                "remove",
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                (shell.clone(),),
            )
            .await?;
        if card.is_null() {
            return Ok(());
        }

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
            .await?;
        let _: bool = jvm
            .invoke_virtual(
                &display,
                "org/kwis/msp/lcdui/Display",
                "removeCard",
                "(Lorg/kwis/msp/lcdui/Card;)Z",
                (card,),
            )
            .await?;

        Ok(())
    }
}
