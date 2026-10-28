use alloc::{string::String, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::Object, util::Hashtable};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Display, Graphics},
    lwc::ShellComponent,
};

// class net.wie.ShellCard: the lcdui card a shown lwc shell sits on. A shell is linked to a card
// and takes the card's events; showing a shell pushes one of these, which has the shell draw itself
// and hands it the keys. The card of each shown shell is kept in a static table
// rather than on the shell, because titles subclass shells and lay their own fields out after the
// SDK's.
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
        jvm.put_field(&mut this, "shell", "Lorg/kwis/msp/lwc/ShellComponent;", shell).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::paint({this:?}, {graphics:?})");

        let shell: ClassInstanceRef<ShellComponent> = jvm.get_field(&this, "shell", "Lorg/kwis/msp/lwc/ShellComponent;").await?;
        // The drawing is the title's: a shell subclass overrides `paint` (LGT 학교가는길) or the
        // specification's `paintContent`; the SDK's components draw nothing of their own here.
        for name in ["paint", "paintContent"] {
            if let Some(class_name) = Self::declaring_class(jvm, &shell, name, "(Lorg/kwis/msp/lcdui/Graphics;)V") {
                return jvm
                    .invoke_virtual(&shell, &class_name, name, "(Lorg/kwis/msp/lcdui/Graphics;)V", (graphics,))
                    .await;
            }
        }

        Ok(())
    }

    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("net.wie.ShellCard::keyNotify({this:?}, {type}, {key})");

        let shell: ClassInstanceRef<ShellComponent> = jvm.get_field(&this, "shell", "Lorg/kwis/msp/lwc/ShellComponent;").await?;

        jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "keyNotify", "(II)Z", (r#type, key))
            .await
    }

    async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, shown: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::showNotify({this:?}, {shown})");

        let shell: ClassInstanceRef<ShellComponent> = jvm.get_field(&this, "shell", "Lorg/kwis/msp/lwc/ShellComponent;").await?;

        jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "showNotify", "(Z)V", (shown,))
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
