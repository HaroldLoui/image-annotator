use ab_glyph::{Font, PxScale};
use egui::Vec2;
use egui::{Pos2, Rect, epaint::EllipseShape};
use image::RgbaImage;
use imageproc::drawing::draw_text_mut;
use tiny_skia::{FillRule, Paint, Path, PathBuilder, PixmapMut, Stroke, Transform};
use tiny_skia::Rect as SkiaRect;

use crate::operators::{Operator, ToolType};

pub trait DrawImage {
    fn draw_on_image<F: Font>(&self, img: &mut RgbaImage, font: &F);
}

impl DrawImage for Operator {
    fn draw_on_image<F: Font>(&self, img: &mut RgbaImage, font: &F) {
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
                draw_text(img, c.center, &n.to_string(), c.radius, font, true);
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

fn draw_text<F: Font>(img: &mut RgbaImage, pos: Pos2, text: &str, size: f32, font: &F, center: bool) {
    let scale = PxScale::from(size);
    let color = image::Rgba([255, 255, 255, 255]);
    
    let (text_x, text_y) = if center {
        // 计算居中位置
        let (tw, th) = crate::font::measure_text(&font, scale, &text);
        let text_x = pos.x - tw / 2.0;
        let text_y = pos.y - th / 2.0;
        (text_x as i32, text_y as i32)
    } else {
        (pos.x as i32, pos.y as i32)
    };

    draw_text_mut(img, color, text_x, text_y, scale, &font, text);
}
