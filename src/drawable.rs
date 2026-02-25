use egui::{Color32, Pos2, Rect, epaint::EllipseShape};
use image::RgbaImage;
use tiny_skia::{FillRule, Paint, Path, PathBuilder, PixmapMut, Stroke, Transform};
use tiny_skia::Rect as SkiaRect;

use crate::operators::{Operator, ToolType};

pub trait DrawImage {
    fn draw_on_image(&self, img: &mut RgbaImage);
}

impl DrawImage for Operator {
    fn draw_on_image(&self, img: &mut RgbaImage) {
        match &self.tool {
            ToolType::Rect(rect) => draw_rect_on_image(self, img, &rect),
            ToolType::Ellipse(ellipse) => draw_ellipse_on_image(self, img, &ellipse),
            ToolType::Arrow(arrow) => draw_points_on_image(self, img, &arrow.points, true),
            ToolType::Line(s, e) => draw_line_on_image(self, img, s, e),
            ToolType::Pencil(points) => draw_points_on_image(self, img, points, false),
            ToolType::Number(_, _) => {},
        }
    }
}

fn draw_rect_on_image(op: &Operator, img: &mut RgbaImage, rect: &Rect) {
    let skia_rect = SkiaRect::from_xywh(rect.left(), rect.top(), rect.width(), rect.height()).unwrap();

    let path = PathBuilder::from_rect(skia_rect);

    draw_skia_image(op, img, &path);
}

fn draw_ellipse_on_image(op: &Operator, img: &mut RgbaImage, ellipse: &EllipseShape) {
    let rect = SkiaRect::from_xywh(
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

fn draw_points_on_image(op: &Operator, img: &mut RgbaImage, points: &Vec<Pos2>, close: bool) {
    if points.is_empty() {
        return;
    }
    let mut pb = PathBuilder::new();
    pb.move_to(points[0].x, points[0].y);
    for p in &points[1..] {
        pb.line_to(p.x, p.y);
    }
    if close {
        pb.close();
    }

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

    if let ToolType::Arrow(ref shape) = op.tool && shape.fill != Color32::TRANSPARENT {
        let fill_color = shape.fill;
        let mut fill_paint = Paint::default();
        fill_paint.set_color_rgba8(fill_color.r(), fill_color.g(), fill_color.b(), fill_color.a());
        fill_paint.anti_alias = true;
        pixmap.fill_path(path, &fill_paint, FillRule::Winding, Transform::identity(), None);
    }

}
