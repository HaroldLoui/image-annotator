use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, epaint::{EllipseShape, PathShape, PathStroke}};

use crate::{AnnotatorApp, StrokeWidth};

#[derive(Clone, Debug, PartialEq)]
pub enum ToolType {
    Rect(Rect),
    Ellipse(EllipseShape),
    Arraw(PathShape),
    Line(Pos2, Pos2),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Operator {
    /// 当前操作的工具类型
    pub tool: ToolType,
    /// 当前操作时的线宽
    pub stroke_width: StrokeWidth,
    /// 当前操作时的颜色
    pub color: Color32,
}

impl Operator {
    pub fn new(tool: ToolType, stroke_width: StrokeWidth, color: Color32) -> Self {
        Operator { tool, stroke_width, color }
    }

    pub fn draw(&self, app: &AnnotatorApp, painter: &Painter) {
        let image_rect = app.last_image_rect.unwrap();
        let zoom = app.zoom * app.display_scale;

        let width = self.stroke_width;
        let color = self.color;
        match &self.tool {
            ToolType::Rect(rect) => {
                let screen_rect = Rect::from_min_max(
                    image_rect.min + rect.min.to_vec2() * zoom,
                    image_rect.min + rect.max.to_vec2() * zoom,
                );
                painter.rect_stroke(
                    screen_rect,
                    0.0,
                    Stroke::new(width, color),
                    StrokeKind::Middle,
                );
            },
            ToolType::Ellipse(ellipse) => {
                let screen_center = image_rect.min + ellipse.center.to_vec2() * zoom;
                let screen_radius = ellipse.radius * zoom;
                
                let screen_ellipse = EllipseShape {
                    center: screen_center,
                    radius: screen_radius,
                    fill: ellipse.fill,
                    stroke: ellipse.stroke,
                };
                painter.add(screen_ellipse);
            },
            ToolType::Arraw(_) => {},
            ToolType::Line(s, e) => {
                let start = image_rect.min + s.to_vec2() * zoom;
                let end = image_rect.min + e.to_vec2() * zoom;
                painter.line(vec![start, end], PathStroke::new(width, color));
            },
        }
    }
}
