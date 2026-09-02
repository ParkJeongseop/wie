use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::Clip;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

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

    async fn refresh(_jvm: &Jvm, context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XDisplay::refresh({x}, {y}, {width}, {height})");

        let platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

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

    // drawImageEx(g, src, dx, dy, mask, sx, sy, w, h, mode): draws the
    // (sx, sy, w, h) region of src at (dx, dy), skipping pixels the mask marks
    // as transparent (mask pixel == black). Used with createMaskableImage.
    #[allow(clippy::too_many_arguments)]
    async fn draw_image_ex(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        src: ClassInstanceRef<Image>,
        dx: i32,
        dy: i32,
        mask: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        w: i32,
        h: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::drawImageEx({graphics:?}, {src:?}, {dx}, {dy}, {mask:?}, {sx}, {sy}, {w}, {h}, {mode})");

        if src.is_null() || graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        let src_image = Image::image(jvm, &src).await?;

        let mut graphics = graphics;
        let dst_image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &dst_image).await?;

        if mask.is_null() {
            // no mask: plain region draw
            canvas.draw(
                dx as _,
                dy as _,
                w as _,
                h as _,
                &*src_image,
                sx,
                sy,
                Clip {
                    x: dx,
                    y: dy,
                    width: w as _,
                    height: h as _,
                },
            );
            return Ok(());
        }

        let mask_image = Image::image(jvm, &mask).await?;

        let dst = canvas.image();
        let (dst_w, dst_h) = (dst.width() as i32, dst.height() as i32);
        let (src_w, src_h) = (src_image.width() as i32, src_image.height() as i32);
        let (mask_w, mask_h) = (mask_image.width() as i32, mask_image.height() as i32);

        for y in 0..h {
            for x in 0..w {
                let (px, py) = (sx + x, sy + y);
                let (qx, qy) = (dx + x, dy + y);
                if px < 0 || py < 0 || px >= src_w || py >= src_h || qx < 0 || qy < 0 || qx >= dst_w || qy >= dst_h {
                    continue;
                }
                if px < mask_w && py < mask_h {
                    let m = mask_image.get_pixel(px, py);
                    if m.r == 0 && m.g == 0 && m.b == 0 {
                        continue; // masked out
                    }
                }
                let color = src_image.get_pixel(px, py);
                canvas.put_pixel(
                    qx,
                    qy,
                    color,
                    Clip {
                        x: 0,
                        y: 0,
                        width: dst_w as _,
                        height: dst_h as _,
                    },
                );
            }
        }

        Ok(())
    }
}
