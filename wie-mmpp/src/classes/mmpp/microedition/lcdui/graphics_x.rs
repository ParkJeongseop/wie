use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

const GRAPHICS: &str = "javax/microedition/lcdui/Graphics";
const GRAPHICS_X: &str = "mmpp/microedition/lcdui/GraphicsX";

// class mmpp.microedition.lcdui.GraphicsX
//
// On an MMPP handset every Graphics is a GraphicsX, and titles downcast the one `paint` receives.
// The alpha and XOR state live in the inherited Graphics fields, which the MIDP drawing code
// applies; this class only adds the MMPP entry points.
//
// XOR mode follows the documented contract (pixels of the current colour become the XOR colour and
// back): a pixel is drawn as `destination ^ colour ^ xorColour`. The inherited `color` field holds
// `colour ^ xorColour` while the mode is on, so the MIDP code's plain XOR produces exactly that.
pub struct GraphicsX;

impl GraphicsX {
    const DEFAULT_ALPHA: i32 = 256;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: GRAPHICS_X,
            parent_class: Some(GRAPHICS),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Image;)V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("setAlpha", "(I)V", Self::set_alpha, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setXORMode", "(I)V", Self::set_xor_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPaintMode", "()V", Self::set_paint_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setColor", "(III)V", Self::set_color_by_rgb, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getColor", "()I", Self::get_color, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPixel", "(II)I", Self::get_pixel, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPixel", "(III)V", Self::set_pixel, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "capture",
                    "(IIII)Ljavax/microedition/lcdui/Image;",
                    Self::capture,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("drawPolyline", "([I[II)V", Self::draw_polyline, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawPolygon", "([I[II)V", Self::draw_polygon, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillPolygon", "([I[II)V", Self::fill_polygon, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("DEFAULT_ALPHA", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("xorColor", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::<clinit>");

        jvm.put_static_field(GRAPHICS_X, "DEFAULT_ALPHA", "I", Self::DEFAULT_ALPHA).await
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::<init>({this:?}, {image:?})");

        jvm.invoke_special(&this, GRAPHICS, "<init>", "(Ljavax/microedition/lcdui/Image;)V", (image,))
            .await
    }

    async fn set_alpha(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, alpha: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::setAlpha({this:?}, {alpha})");

        if !(0..=Self::DEFAULT_ALPHA).contains(&alpha) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "alpha out of range").await);
        }

        // MMPP counts 0..=256, the MIDP layer 0..=255
        jvm.put_field(&mut this, "alpha", "I", alpha * 255 / Self::DEFAULT_ALPHA).await
    }

    async fn set_xor_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, rgb: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::setXORMode({this:?}, {rgb:#x})");

        let color = Self::paint_color(jvm, &this).await?;
        let xor_color = rgb & 0xffffff;

        jvm.put_field(&mut this, "xorColor", "I", xor_color).await?;
        jvm.put_field(&mut this, "xorMode", "Z", true).await?;
        jvm.put_field(&mut this, "color", "I", color ^ xor_color).await?;
        // entering XOR mode puts the alpha back to its default
        jvm.put_field(&mut this, "alpha", "I", 255).await
    }

    async fn set_paint_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::setPaintMode({this:?})");

        let color = Self::paint_color(jvm, &this).await?;

        jvm.put_field(&mut this, "xorMode", "Z", false).await?;
        jvm.put_field(&mut this, "color", "I", color).await
    }

    /// The colour the application chose, whichever mode is on.
    async fn paint_color(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        let color: i32 = jvm.get_field(this, "color", "I").await?;

        Ok(color ^ Self::xor_mask(jvm, this).await?)
    }

    async fn xor_mask(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        if jvm.get_field(this, "xorMode", "Z").await? {
            jvm.get_field(this, "xorColor", "I").await
        } else {
            Ok(0)
        }
    }

    async fn set_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, rgb: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::setColor({this:?}, {rgb:#x})");

        let stored = rgb ^ Self::xor_mask(jvm, &this).await?;

        jvm.invoke_special(&this, GRAPHICS, "setColor", "(I)V", (stored,)).await
    }

    async fn set_color_by_rgb(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, r: i32, g: i32, b: i32) -> JvmResult<()> {
        Self::set_color(jvm, context, this, ((r & 0xff) << 16) | ((g & 0xff) << 8) | (b & 0xff)).await
    }

    async fn get_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::getColor({this:?})");

        Self::paint_color(jvm, &this).await
    }

    fn as_graphics(this: &ClassInstanceRef<Self>) -> ClassInstanceRef<Graphics> {
        ClassInstanceRef::new(this.instance.clone())
    }

    async fn translation(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<(i32, i32)> {
        Ok((
            jvm.get_field(this, "translateX", "I").await?,
            jvm.get_field(this, "translateY", "I").await?,
        ))
    }

    async fn get_pixel(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<i32> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::getPixel({this:?}, {x}, {y})");

        let (translate_x, translate_y) = Self::translation(jvm, &this).await?;
        let (x, y) = (translate_x + x, translate_y + y);

        let image = Graphics::image(jvm, &mut Self::as_graphics(&this)).await?;
        let image = Image::image(jvm, &image).await?;
        if x < 0 || y < 0 || x as u32 >= image.width() || y as u32 >= image.height() {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "pixel outside the drawing area")
                .await);
        }
        let color = image.get_pixel(x, y);

        Ok(((color.r as i32) << 16) | ((color.g as i32) << 8) | color.b as i32)
    }

    async fn set_pixel(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, rgb: i32) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::setPixel({this:?}, {x}, {y}, {rgb:#x})");

        // the pixel takes exactly this colour, whatever colour, mode and alpha are current
        let color: i32 = jvm.get_field(&this, "color", "I").await?;
        let xor_mode: bool = jvm.get_field(&this, "xorMode", "Z").await?;
        let alpha: i32 = jvm.get_field(&this, "alpha", "I").await?;

        jvm.put_field(&mut this, "color", "I", rgb & 0xffffff).await?;
        jvm.put_field(&mut this, "xorMode", "Z", false).await?;
        jvm.put_field(&mut this, "alpha", "I", 255).await?;
        let result: JvmResult<()> = jvm.invoke_virtual(&this, GRAPHICS, "fillRect", "(IIII)V", (x, y, 1, 1)).await;

        jvm.put_field(&mut this, "color", "I", color).await?;
        jvm.put_field(&mut this, "xorMode", "Z", xor_mode).await?;
        jvm.put_field(&mut this, "alpha", "I", alpha).await?;

        result
    }

    async fn capture(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::capture({this:?}, {x}, {y}, {width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "empty capture area").await);
        }

        let (translate_x, translate_y) = Self::translation(jvm, &this).await?;
        let source = Graphics::image(jvm, &mut Self::as_graphics(&this)).await?;

        let captured: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;
        let target: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(
                &captured,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        const TOP_LEFT: i32 = 16 | 4;
        let _: () = jvm
            .invoke_virtual(
                &target,
                GRAPHICS,
                "drawImage",
                "(Ljavax/microedition/lcdui/Image;III)V",
                (source, -(translate_x + x), -(translate_y + y), TOP_LEFT),
            )
            .await?;

        Ok(captured)
    }

    async fn points(jvm: &Jvm, xs: &ClassInstanceRef<Array<i32>>, ys: &ClassInstanceRef<Array<i32>>, count: i32) -> JvmResult<Vec<(i32, i32)>> {
        let count = count.max(0) as usize;
        let xs: Vec<i32> = jvm.load_array(xs, 0, count).await?;
        let ys: Vec<i32> = jvm.load_array(ys, 0, count).await?;

        Ok(xs.into_iter().zip(ys).collect())
    }

    async fn draw_segments(jvm: &Jvm, this: &ClassInstanceRef<Self>, points: &[(i32, i32)]) -> JvmResult<()> {
        for segment in points.windows(2) {
            let _: () = jvm
                .invoke_virtual(
                    this,
                    GRAPHICS,
                    "drawLine",
                    "(IIII)V",
                    (segment[0].0, segment[0].1, segment[1].0, segment[1].1),
                )
                .await?;
        }

        Ok(())
    }

    async fn draw_polyline(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        xs: ClassInstanceRef<Array<i32>>,
        ys: ClassInstanceRef<Array<i32>>,
        count: i32,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::drawPolyline({this:?}, {xs:?}, {ys:?}, {count})");

        let points = Self::points(jvm, &xs, &ys, count).await?;

        Self::draw_segments(jvm, &this, &points).await
    }

    async fn draw_polygon(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        xs: ClassInstanceRef<Array<i32>>,
        ys: ClassInstanceRef<Array<i32>>,
        count: i32,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::drawPolygon({this:?}, {xs:?}, {ys:?}, {count})");

        let mut points = Self::points(jvm, &xs, &ys, count).await?;
        if let Some(first) = points.first().copied() {
            points.push(first);
        }

        Self::draw_segments(jvm, &this, &points).await
    }

    async fn fill_polygon(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        xs: ClassInstanceRef<Array<i32>>,
        ys: ClassInstanceRef<Array<i32>>,
        count: i32,
    ) -> JvmResult<()> {
        tracing::debug!("mmpp.microedition.lcdui.GraphicsX::fillPolygon({this:?}, {xs:?}, {ys:?}, {count})");

        let points = Self::points(jvm, &xs, &ys, count).await?;
        let (Some(top), Some(bottom)) = (points.iter().map(|point| point.1).min(), points.iter().map(|point| point.1).max()) else {
            return Ok(());
        };

        // even-odd scanline fill: one span per pair of edge crossings on each row
        for y in top..=bottom {
            let mut crossings = Vec::new();
            for index in 0..points.len() {
                let (x1, y1) = points[index];
                let (x2, y2) = points[(index + 1) % points.len()];
                if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                    crossings.push(x1 + ((y - y1) as i64 * (x2 - x1) as i64 / (y2 - y1) as i64) as i32);
                }
            }
            crossings.sort_unstable();

            for span in crossings.chunks_exact(2) {
                let _: () = jvm
                    .invoke_virtual(&this, GRAPHICS, "fillRect", "(IIII)V", (span[0], y, span[1] - span[0] + 1, 1))
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstance, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};
    use wie_util::Result;

    use super::{GRAPHICS, GRAPHICS_X};

    /// A white 4x4 image and the Graphics the LCDUI hands out for it on an MMPP platform.
    async fn white_canvas(jvm: &Jvm) -> JvmResult<(ClassInstanceRef<Image>, ClassInstanceRef<Graphics>)> {
        let key = JavaLangString::from_rust_string(jvm, "wie.midp.graphics").await?;
        let value = JavaLangString::from_rust_string(jvm, GRAPHICS_X).await?;
        let _: Option<Box<dyn ClassInstance>> = jvm
            .invoke_static(
                "java/lang/System",
                "setProperty",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                (key, value),
            )
            .await?;

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (4, 4),
            )
            .await?;
        let graphics: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "setColor", "(I)V", (0xffffff,)).await?;
        let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (0, 0, 4, 4)).await?;

        Ok((image, graphics))
    }

    async fn rgb(jvm: &Jvm, image: &ClassInstanceRef<Image>, x: i32, y: i32) -> JvmResult<(u8, u8, u8)> {
        let color = Image::image(jvm, image).await?.get_pixel(x, y);

        Ok((color.r, color.g, color.b))
    }

    fn protos() -> Box<[Box<[wie_jvm_support::WieJavaClassProto]>]> {
        Box::new([wie_midp::get_protos().into(), crate::get_protos().into()])
    }

    #[test]
    fn every_graphics_is_a_graphics_x() -> Result<()> {
        run_jvm_test(protos(), |jvm| async move {
            let (_, graphics) = white_canvas(&jvm).await?;

            assert!(jvm.is_instance(&**graphics.instance.as_ref().unwrap(), GRAPHICS_X));

            Ok(())
        })
    }

    #[test]
    fn alpha_blends_until_it_is_set_back() -> Result<()> {
        run_jvm_test(protos(), |jvm| async move {
            let (image, graphics) = white_canvas(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "setColor", "(I)V", (0x000000,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS_X, "setAlpha", "(I)V", (128,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (0, 0, 1, 1)).await?;
            let default_alpha: i32 = jvm.get_static_field(GRAPHICS_X, "DEFAULT_ALPHA", "I").await?;
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS_X, "setAlpha", "(I)V", (default_alpha,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (1, 0, 1, 1)).await?;

            let (half, _, _) = rgb(&jvm, &image, 0, 0).await?;
            assert!((120..=136).contains(&half), "half-transparent black over white gave {half}");
            assert_eq!(rgb(&jvm, &image, 1, 0).await?, (0, 0, 0));

            let out_of_range: JvmResult<()> = jvm.invoke_virtual(&graphics, GRAPHICS_X, "setAlpha", "(I)V", (257,)).await;
            assert!(out_of_range.is_err());

            Ok(())
        })
    }

    #[test]
    fn xor_mode_swaps_the_current_and_the_xor_colour() -> Result<()> {
        run_jvm_test(protos(), |jvm| async move {
            let (image, graphics) = white_canvas(&jvm).await?;

            // current colour red, XOR colour white: white pixels turn red, a second pass restores them
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "setColor", "(I)V", (0xff0000,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS_X, "setXORMode", "(I)V", (0xffffff,)).await?;
            let color: i32 = jvm.invoke_virtual(&graphics, GRAPHICS, "getColor", "()I", ()).await?;
            assert_eq!(color, 0xff0000);

            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (0, 0, 2, 1)).await?;
            assert_eq!(rgb(&jvm, &image, 0, 0).await?, (0xff, 0, 0));
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (0, 0, 1, 1)).await?;
            assert_eq!(rgb(&jvm, &image, 0, 0).await?, (0xff, 0xff, 0xff));

            // a colour chosen while the mode is on behaves the same way
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "setColor", "(I)V", (0x0000ff,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (2, 0, 1, 1)).await?;
            assert_eq!(rgb(&jvm, &image, 2, 0).await?, (0, 0, 0xff));

            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS_X, "setPaintMode", "()V", ()).await?;
            let color: i32 = jvm.invoke_virtual(&graphics, GRAPHICS, "getColor", "()I", ()).await?;
            assert_eq!(color, 0x0000ff);
            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "fillRect", "(IIII)V", (3, 0, 1, 1)).await?;
            assert_eq!(rgb(&jvm, &image, 3, 0).await?, (0, 0, 0xff));

            Ok(())
        })
    }

    #[test]
    fn pixels_and_capture_read_what_was_drawn() -> Result<()> {
        run_jvm_test(protos(), |jvm| async move {
            let (_, graphics) = white_canvas(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, GRAPHICS_X, "setPixel", "(III)V", (2, 1, 0x123456)).await?;
            let pixel: i32 = jvm.invoke_virtual(&graphics, GRAPHICS_X, "getPixel", "(II)I", (2, 1)).await?;
            assert_eq!(pixel, 0x123456);
            let outside: JvmResult<i32> = jvm.invoke_virtual(&graphics, GRAPHICS_X, "getPixel", "(II)I", (4, 0)).await;
            assert!(outside.is_err());

            let captured: ClassInstanceRef<Image> = jvm
                .invoke_virtual(&graphics, GRAPHICS_X, "capture", "(IIII)Ljavax/microedition/lcdui/Image;", (2, 1, 2, 2))
                .await?;
            assert_eq!(rgb(&jvm, &captured, 0, 0).await?, (0x12, 0x34, 0x56));
            assert_eq!(rgb(&jvm, &captured, 1, 1).await?, (0xff, 0xff, 0xff));

            Ok(())
        })
    }
}
