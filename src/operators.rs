use egui::{Color32, Painter, Rect, Stroke, StrokeKind};
use image::{Rgba, RgbaImage};

use crate::LineWidth;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToolType {
    Rect(Rect),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Operator {
    /// 当前操作的工具类型
    tool: ToolType,
    /// 当前操作时的线宽
    line_width: LineWidth,
    /// 当前操作时的颜色
    color: Color32,
}

impl Operator {
    pub fn new(tool: ToolType, line_width: LineWidth, color: Color32) -> Self {
        Operator { tool, line_width, color }
    }

    pub fn draw(&self, painter: &Painter) {
        match self.tool {
            ToolType::Rect(rect) => {
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(self.line_width, self.color),
                    StrokeKind::Middle,
                );
            },
        }
    }

    pub fn draw_process(&self, painter: &Painter) {
        match self.tool {
            ToolType::Rect(rect) => {
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(self.line_width, self.color),
                    StrokeKind::Middle,
                );
            },
        }
    }

    pub fn draw_on_image(&self, app: &crate::AnnotatorApp, img: &mut RgbaImage) {
        match self.tool {
            ToolType::Rect(rect) => self.draw_rect_on_image(app, img, &rect),
        }
    }

    fn draw_rect_on_image(&self, app: &crate::AnnotatorApp, img: &mut RgbaImage, rect: &Rect) {
        let color = self.color.to_srgba_unmultiplied().into();

        let (img_w, img_h) = img.dimensions();

        let inv = 1.0 / app.display_scale;
        let rect = rect.scale_from_center(inv);

        let min_x = rect.min.x.max(0.0) as i32;
        let min_y = rect.min.y.max(0.0) as i32;
        let max_x = rect.max.x.min(img_w as f32 - 1.0) as i32;
        let max_y = rect.max.y.min(img_h as f32 - 1.0) as i32;

        let w: f32 = self.line_width.into();
        let thickness = (w / app.display_scale).round() as u32;
        let t = thickness as i32;

        // 画上边和下边
        for dy in 0..t {
            for x in min_x..=max_x {
                let y_top = min_y + dy;
                let y_bottom = max_y - dy;

                if y_top >= 0 && y_top < img_h as i32 {
                    img.put_pixel(x as u32, y_top as u32, color);
                }

                if y_bottom >= 0 && y_bottom < img_h as i32 {
                    img.put_pixel(x as u32, y_bottom as u32, color);
                }
            }
        }

        // 画左边和右边
        for dx in 0..t {
            for y in min_y..=max_y {
                let x_left = min_x + dx;
                let x_right = max_x - dx;

                if x_left >= 0 && x_left < img_w as i32 {
                    img.put_pixel(x_left as u32, y as u32, color);
                }

                if x_right >= 0 && x_right < img_w as i32 {
                    img.put_pixel(x_right as u32, y as u32, color);
                }
            }
        }
    }
}
