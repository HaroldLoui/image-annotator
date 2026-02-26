use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use egui::Vec2;
use egui::{Pos2, Rect, epaint::EllipseShape};
use image::RgbaImage;
use imageproc::drawing::draw_text_mut;
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
            ToolType::Number(c, n) => {
                let ellipse = EllipseShape {
                    radius: Vec2::new(c.radius, c.radius),
                    center: c.center,
                    fill: c.fill,
                    stroke: c.stroke,
                };
                draw_ellipse_on_image(self, img, &ellipse);
                draw_text(img, c.center.into(), &n.to_string(), c.radius);
            },
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

    if let Some(fill_color) = op.fill_color {
        let mut fill_paint = Paint::default();
        fill_paint.set_color_rgba8(fill_color.r(), fill_color.g(), fill_color.b(), fill_color.a());
        fill_paint.anti_alias = true;
        pixmap.fill_path(path, &fill_paint, FillRule::Winding, Transform::identity(), None);
    }

}

fn draw_text(img: &mut RgbaImage, pos: (f32, f32), text: &str, size: f32) {

    let font_path = "C:/Windows/Fonts/HarmonyOS_Sans_SC_Regular.ttf";
    let data = std::fs::read(font_path).unwrap();
    let font = FontRef::try_from_slice(&data).unwrap();
    let scale = PxScale::from(size);
    let color = image::Rgba([255, 255, 255, 255]);
    
    let (tw, th) = measure_text(&font, scale, &text);
    let text_x = pos.0 - tw / 2.0;
    let text_y = pos.1 - th / 2.0;

    draw_text_mut(img, color, text_x as i32, text_y as i32, scale, &font, text);
}

fn measure_text(font: &FontRef, scale: PxScale, text: &str) -> (f32, f32) {
    let scaled = font.as_scaled(scale);
    let width: f32 = text.chars()
        .map(|c| scaled.h_advance(font.glyph_id(c)))
        .sum();
    let height = scaled.ascent() - scaled.descent();
    (width, height)
}
