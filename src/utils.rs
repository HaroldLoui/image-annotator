use egui::{Pos2, Rect, Vec2};

use crate::AnnotatorApp;

#[derive(Debug, Clone, Copy)]
pub struct AppHelper {
    pub image_rect_min: Pos2,
    pub image_size: Vec2,
    pub zoom: f32,
    pub display_scale: f32,
}

impl AppHelper {
    pub fn from_app(app: &AnnotatorApp) -> Self {
        AppHelper {
            image_rect_min: app.last_image_rect.map_or(Pos2::default(), |r| r.min),
            image_size: app.image_size,
            zoom: app.zoom,
            display_scale: app.display_scale,
        }
    }

    pub fn get_image_rect(&self) -> Rect {
        Rect::from_min_size(self.image_rect_min, self.image_size * self.zoom)
    }

    pub fn screen_to_image(&self, pos: Pos2, image_rect: Option<Rect>) -> Pos2 {
        let image_rect_min = image_rect.map_or(self.image_rect_min, |r| r.min);
        (pos - image_rect_min).to_pos2() / (self.zoom * self.display_scale)
    }

    pub fn image_to_screen(&self, pos: Pos2) -> Pos2 {
        self.image_rect_min + pos.to_vec2() * self.zoom * self.display_scale
    }
}
