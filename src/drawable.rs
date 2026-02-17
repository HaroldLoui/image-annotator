use std::f32::consts::TAU;

use egui::{Rect, epaint::EllipseShape};
use image::{Rgba, RgbaImage};
use imageproc::{drawing::draw_antialiased_line_segment_mut, pixelops::interpolate};

use crate::{
    AnnotatorApp,
    operators::{Operator, ToolType},
};

pub trait Draw {
    fn draw_on_image(&self, app: &AnnotatorApp, img: &mut RgbaImage);
}

impl Draw for Operator {
    fn draw_on_image(&self, app: &AnnotatorApp, img: &mut RgbaImage) {
        match self.tool {
            ToolType::Rect(rect) => draw_rect_on_image(self, app, img, &rect),
            ToolType::Ellipse(ellipse) => draw_ellipse_on_image(self, app, img, &ellipse),
        }
    }
}

fn draw_rect_on_image(op: &Operator, app: &AnnotatorApp, img: &mut RgbaImage, rect: &Rect) {
    let color = op.color.to_srgba_unmultiplied().into();

    let (img_w, img_h) = img.dimensions();

    let inv = 1.0 / app.display_scale;
    let rect = Rect::from_min_max(
        rect.min * inv,
        rect.max * inv,
    );

    let w: f32 = op.stroke_width.into();
    let thickness = (w / app.display_scale).round() as u32;

    let min_x = rect.min.x;
    let min_y = rect.min.y;
    let max_x = rect.max.x;
    let max_y = rect.max.y;

    for offset in 0..thickness {
        let o = offset as f32;

        // 上边
        draw_antialiased_line_segment_mut(
            img,
            (min_x as i32, (min_y + o) as i32),
            (max_x as i32, (min_y + o) as i32),
            color,
            interpolate,
        );

        // 下边
        draw_antialiased_line_segment_mut(
            img,
            (min_x as i32, (max_y - o) as i32),
            (max_x as i32, (max_y - o) as i32),
            color,
            interpolate,
        );

        // 左边
        draw_antialiased_line_segment_mut(
            img,
            ((min_x + o) as i32, min_y as i32),
            ((min_x + o) as i32, max_y as i32),
            color,
            interpolate,
        );

        // 右边
        draw_antialiased_line_segment_mut(
            img,
            ((max_x - o) as i32, min_y as i32),
            ((max_x - o) as i32, max_y as i32),
            color,
            interpolate,
        );
    }
}

fn draw_ellipse_on_image(
    op: &Operator,
    app: &AnnotatorApp,
    img: &mut RgbaImage,
    ellipse: &EllipseShape,
) {
    let color = image::Rgba(op.color.to_srgba_unmultiplied());

    let inv = 1.0 / app.display_scale;

    let rect = Rect::from_min_max(
        (ellipse.center - ellipse.radius) * inv,
        (ellipse.center + ellipse.radius) * inv,
    );

    // 再重新计算中心和半径
    let cx = (rect.min.x + rect.max.x) / 2.0;
    let cy = (rect.min.y + rect.max.y) / 2.0;

    let rx = rect.width() / 2.0;
    let ry = rect.height() / 2.0;

    let sw: f32 = op.stroke_width.into();
    let stroke = (sw / app.display_scale).max(1.0);

    let outer_rx = rx + stroke / 2.0;
    let outer_ry = ry + stroke / 2.0;

    let inner_rx = (rx - stroke / 2.0).max(0.0);
    let inner_ry = (ry - stroke / 2.0).max(0.0);

    let min_x = (cx - outer_rx).floor() as i32;
    let max_x = (cx + outer_rx).ceil() as i32;
    let min_y = (cy - outer_ry).floor() as i32;
    let max_y = (cy + outer_ry).ceil() as i32;

    // 4 个子采样点
    let samples = [
        (0.25, 0.25),
        (0.75, 0.25),
        (0.25, 0.75),
        (0.75, 0.75),
    ];

    for y in min_y..=max_y {
        for x in min_x..=max_x {

            if x < 0 || y < 0 ||
               x >= img.width() as i32 ||
               y >= img.height() as i32 {
                continue;
            }

            let mut coverage = 0;

            for (sx, sy) in samples {

                let fx = x as f32 + sx;
                let fy = y as f32 + sy;

                let dx = fx - cx;
                let dy = fy - cy;

                let outer =
                    (dx * dx) / (outer_rx * outer_rx) +
                    (dy * dy) / (outer_ry * outer_ry);

                let inner =
                    (dx * dx) / (inner_rx * inner_rx) +
                    (dy * dy) / (inner_ry * inner_ry);

                if outer <= 1.0 && inner >= 1.0 {
                    coverage += 1;
                }
            }

            if coverage > 0 {
                let alpha = (coverage as f32 / 4.0) * 255.0;

                let existing = img.get_pixel(x as u32, y as u32);
                let new_pixel = blend_pixel(*existing, color, alpha as u8);

                img.put_pixel(x as u32, y as u32, new_pixel);
            }
        }
    }
}
fn blend_pixel(
    dst: image::Rgba<u8>,
    src: image::Rgba<u8>,
    alpha: u8,
) -> image::Rgba<u8> {

    let a = alpha as f32 / 255.0;

    let r = src[0] as f32 * a + dst[0] as f32 * (1.0 - a);
    let g = src[1] as f32 * a + dst[1] as f32 * (1.0 - a);
    let b = src[2] as f32 * a + dst[2] as f32 * (1.0 - a);

    image::Rgba([r as u8, g as u8, b as u8, 255])
}

