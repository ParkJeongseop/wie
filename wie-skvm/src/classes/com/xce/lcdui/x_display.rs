use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::{Clip, Color};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::{
    lcdui::{Graphics, Image},
    midlet::MIDlet,
};

use crate::classes::com::skt::m::Graphics2D;

// class com.xce.lcdui.XDisplay
pub struct XDisplay;

impl XDisplay {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XDisplay",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("refresh", "(IIII)V", Self::refresh, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "clear",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;II)V",
                    Self::clear,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "copyLCD",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IIII)V",
                    Self::copy_lcd,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "drawImageEx",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                    Self::draw_image_ex,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("width", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height2", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::<clinit>()");

        // TODO: temp
        jvm.put_static_field("com/xce/lcdui/XDisplay", "width", "I", 240).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "height", "I", 320).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "height2", "I", 320).await?;

        Ok(())
    }

    async fn refresh(jvm: &Jvm, context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::refresh({x}, {y}, {width}, {height})");

        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        let display = MIDlet::display(jvm, &midlet).await?;
        let screen_image: ClassInstanceRef<Image> = jvm
            .get_field(
                &display,
                "javax/microedition/lcdui/Display",
                "screenImage",
                "Ljavax/microedition/lcdui/Image;",
            )
            .await?;
        let image = Image::image(jvm, &screen_image).await?;
        let platform = context.system().platform();
        let screen = platform.screen();
        screen.paint(&*image);

        Ok(())
    }

    // clear(g, image, x, y): blacks out what the graphics draws on from (x, y), over the size of the
    // image when one is given. The one caller (코인마스터) passes a null image and the origin, then
    // draws white text on what it cleared.
    async fn clear(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::clear({graphics:?}, {image:?}, {x}, {y})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let mut graphics = graphics;
        let target = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &target).await?;
        let (target_width, target_height) = (canvas.image().width(), canvas.image().height());

        let (width, height) = if image.is_null() {
            (target_width, target_height)
        } else {
            let image = Image::image(jvm, &image).await?;
            (image.width(), image.height())
        };

        canvas.fill_rect(
            x,
            y,
            width,
            height,
            Color { a: 0xff, r: 0, g: 0, b: 0 },
            Clip {
                x: 0,
                y: 0,
                width: target_width,
                height: target_height,
            },
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn copy_lcd(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XDisplay::copyLCD({graphics:?}, {image:?}, {x}, {y}, {width}, {height})",);

        Ok(())
    }

    // drawImageEx(g, image, tx, ty, src, sx, sy, sw, sh, mode): draws the (sx, sy, sw, sh) region of
    // `src` — the fifth argument — at (tx, ty) with a Graphics2D draw mode, as the vendor's own
    // Java did (`Graphics2D.getGraphics2D(g).drawImage(tx, ty, src, sx, sy, sw, sh, mode)`); the
    // second image is not used. The one title that calls this (다크슬레이어2) passes null there and
    // its 240x1 strip as the source; taking the second image for the source threw at every row.
    #[allow(clippy::too_many_arguments)]
    async fn draw_image_ex(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        tx: i32,
        ty: i32,
        src: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::drawImageEx({graphics:?}, {image:?}, {tx}, {ty}, {src:?}, {sx}, {sy}, {sw}, {sh}, {mode})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }
        // A title that passes its source second and nothing fifth still draws.
        let src = if src.is_null() { image } else { src };

        let graphics_2d: ClassInstanceRef<Graphics2D> = jvm
            .invoke_static(
                "com/skt/m/Graphics2D",
                "getGraphics2D",
                "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                (graphics,),
            )
            .await?;
        jvm.invoke_virtual(
            &graphics_2d,
            "com/skt/m/Graphics2D",
            "drawImage",
            "(IILjavax/microedition/lcdui/Image;IIIII)V",
            (tx, ty, src, sx, sy, sw, sh, mode),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicU32, Ordering};

    use test_utils::{TestPlatform, run_jvm_test_with_system};

    use super::*;

    #[test]
    fn refresh_presents_shared_pixels_without_consuming_pending_repaint() {
        let midlet_proto = WieJavaClassProto {
            name: "test/RefreshMidlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let paints = Arc::new(AtomicU32::new(0));
        let observed_paints = paints.clone();
        let platform = TestPlatform::with_paint_handler(move |image| {
            let pixel = image.get_pixel(3, 4);
            assert_eq!((pixel.r, pixel.g, pixel.b), (0x12, 0x34, 0x56));
            observed_paints.fetch_add(1, Ordering::SeqCst);
        });
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), [XDisplay::as_proto(), midlet_proto].into()]),
            Box::new(platform),
            move |jvm, _system| async move {
                let midlet: ClassInstanceRef<MIDlet> = jvm.instantiate_class("test/RefreshMidlet").await?.into();
                let _: () = jvm
                    .invoke_special(&midlet, "javax/microedition/midlet/MIDlet", "<init>", "()V", ())
                    .await?;
                let display = MIDlet::display(&jvm, &midlet).await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "getScreenGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (3, 4, 2, 1))
                    .await?;
                for pending in [false, true] {
                    if pending {
                        let _: () = jvm
                            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, 320, 240))
                            .await?;
                    }
                    let _: () = jvm.invoke_static("com/xce/lcdui/XDisplay", "refresh", "(IIII)V", (3, 4, 2, 1)).await?;
                    assert_eq!(paints.load(Ordering::SeqCst), if pending { 2 } else { 1 });
                    assert_eq!(
                        jvm.get_field::<bool>(&display, "javax/microedition/lcdui/Display", "repaintPending", "Z")
                            .await?,
                        pending
                    );
                }
                Ok(())
            },
        )
        .unwrap();
    }
}
