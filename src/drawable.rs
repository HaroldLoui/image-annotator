use egui::{Pos2, Rect, epaint::EllipseShape};
use image::RgbaImage;
use tiny_skia::{Paint, Path, PathBuilder, PixmapMut, Stroke, Transform};

use crate::operators::{Operator, ToolType};

pub trait Draw {
    fn draw_on_image(&self, img: &mut RgbaImage);
}

impl Draw for Operator {
    fn draw_on_image(&self, img: &mut RgbaImage) {
        match &self.tool {
            ToolType::Rect(rect) => draw_rect_on_image(self, img, &rect),
            ToolType::Ellipse(ellipse) => draw_ellipse_on_image(self, img, &ellipse),
            ToolType::Arraw(_path_shape) => todo!(),
            ToolType::Line(s, e) => draw_line_on_image(self, img, s, e),
        }
    }
}

fn draw_rect_on_image(op: &Operator, img: &mut RgbaImage, rect: &Rect) {
    let skia_rect =
        tiny_skia::Rect::from_xywh(rect.left(), rect.top(), rect.width(), rect.height()).unwrap();

    let path = PathBuilder::from_rect(skia_rect);

    draw_skia_image(op, img, &path);
}

fn draw_ellipse_on_image(op: &Operator, img: &mut RgbaImage, ellipse: &EllipseShape) {
    let rect = tiny_skia::Rect::from_xywh(
        ellipse.center.x - ellipse.radius.x,
        ellipse.center.y - ellipse.radius.y,
        ellipse.radius.x * 2.0,
        ellipse.radius.y * 2.0,
    )
    .unwrap();

    let path = PathBuilder::from_oval(rect).unwrap();

    draw_skia_image(op, img, &path);
}

fn draw_line_on_image(op: &Operator, img: &mut RgbaImage, start: &Pos2, end: &Pos2) {
    // stroke.line_cap = LineCap::Square;

    let mut pb = PathBuilder::new();
    pb.move_to(start.x, start.y);
    pb.line_to(end.x, end.y);

    if let Some(path) = pb.finish() {
        draw_skia_image(op, img, &path);
    }
}

fn draw_skia_image(op: &Operator, img: &mut RgbaImage, path: &Path) {
    let (width, height) = img.dimensions();

    // 直接用 RgbaImage 的 buffer 构造 PixmapMut，零拷贝
    let mut pixmap = PixmapMut::from_bytes(img.as_mut(), width, height).unwrap();

    let color = op.color;
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = op.stroke_width.into();

    pixmap.stroke_path(path, &paint, &stroke, Transform::identity(), None);
}
