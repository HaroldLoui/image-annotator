#![allow(unused)]

use eframe::{App, egui};
use egui::{Color32, ColorImage, Image, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView};

mod color_picker;
mod toolbar;

use toolbar::Tool;
use color_picker::ColorPickerButton;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Annotator",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(AnnotatorApp::new(cc))
        )}),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineWidth {
    ONE,
    THREE,
    FIVE,
    Custom(f32)
}

impl Into<f32> for LineWidth {
    fn into(self) -> f32 {
        match self {
            LineWidth::ONE => 1f32,
            LineWidth::THREE => 3f32,
            LineWidth::FIVE => 5f32,
            LineWidth::Custom(x) => x,
        }
    }
}

struct AnnotatorApp {
    texture: Option<TextureHandle>,
    image_size: Vec2,
    image_path: Option<String>,
    current_tool: Tool,
    current_color: Color32,
    line_width: LineWidth,
    start_pos: Option<Pos2>,
    rectangles: Vec<Rect>,
    color_picker: ColorPickerButton,
    display_scale: f32,
    zoom: f32,
    pan: Vec2,
    last_image_rect_min: Option<Pos2>,
}

impl AnnotatorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 从命令行读取图片路径
        let args: Vec<String> = std::env::args().collect();
        let mut texture = None;
        let mut image_size = egui::Vec2::ZERO;
        let mut image_path = None;
        let mut display_scale = 1.0;

        if args.len() > 1 {
            image_path = Some(args[1].clone());
            let (color_image, scale) = scale_color_image(&args[1], &mut image_size);
            display_scale = scale;
            texture = Some(cc.egui_ctx.load_texture(
                "loaded_image",
                color_image,
                Default::default(),
            ));
        }

        Self {
            texture,
            image_size,
            image_path,
            current_tool: Tool::Rectangle,
            current_color: Color32::WHITE,
            line_width: LineWidth::THREE,
            start_pos: None,
            rectangles: Vec::new(),
            color_picker: ColorPickerButton::new("ColorPicker", Color32::WHITE),
            display_scale,
            zoom: 1.0,
            pan: Vec2::ZERO,
            last_image_rect_min: None,
        }
    }

    fn save_image(&self, ctx: &egui::Context) {
        if let Some(path) = &self.image_path {
            let mut img = image::open(path)
                .expect("Failed to reopen image")
                .to_rgba8();

            for rect in &self.rectangles {
                self.draw_rect_on_image(&mut img, rect);
            }

            img.save(path).expect("Failed to save image");
            println!("image saved!");

            // let _ = std::process::Command::new("wl-copy")
            //     .arg("--type")
            //     .arg("image/png")
            //     .arg(path)
            //     .spawn();

            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn draw_rect_on_image(&self, img: &mut image::RgbaImage, rect: &Rect) {
        let color = image::Rgba([255, 0, 0, 255]); // 红色

        let inv_scale = 1.0 / self.display_scale;

        let min_x = (rect.min.x * inv_scale) as u32;
        let min_y = (rect.min.y * inv_scale) as u32;
        let max_x = (rect.max.x * inv_scale) as u32;
        let max_y = (rect.max.y * inv_scale) as u32;

        // 上下边
        for x in min_x..max_x {
            if min_y < img.height() {
                img.put_pixel(x, min_y, color);
            }
            if max_y < img.height() {
                img.put_pixel(x, max_y, color);
            }
        }

        // 左右边
        for y in min_y..max_y {
            if min_x < img.width() {
                img.put_pixel(min_x, y, color);
            }
            if max_x < img.width() {
                img.put_pixel(max_x, y, color);
            }
        }
    }

    fn screen_to_image(
        &self,
        pos: egui::Pos2,
        image_rect: egui::Rect,
    ) -> egui::Pos2 {
        (Pos2::ZERO + (pos - image_rect.min)) / self.zoom
    }

    fn reset_view(&mut self, available_rect: egui::Rect) {
        self.zoom = 1.0;

        // 居中图片
        let image_size = self.texture.as_ref().unwrap().size_vec2();

        let center_offset =
            (available_rect.size() - image_size) / 2.0;

        self.pan = center_offset;
    }


}

fn scale_color_image(path: &str, image_size: &mut Vec2) -> (ColorImage, f32) {

    let img = image::open(path).expect("Failed to open image").to_rgba8();
    let (w, h) = img.dimensions();

    let max_size = 2048u32;

    let scale = if w > max_size || h > max_size {
        let scale_w = max_size as f32 / w as f32;
        let scale_h = max_size as f32 / h as f32;
        scale_w.min(scale_h)
    } else {
        1.0
    };

    let display_img = if scale < 1.0 {
        image::imageops::resize(
            &img,
            (w as f32 * scale) as u32,
            (h as f32 * scale) as u32,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img.clone()
    };

    let size = display_img.dimensions();
    *image_size = egui::vec2(size.0 as f32, size.1 as f32);

    let rgba = display_img.into_raw();
    let color_image = ColorImage::from_rgba_unmultiplied(
        [size.0 as usize, size.1 as usize],
        &rgba,
    );
    (color_image, scale)
}

impl App for AnnotatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理缩放（Ctrl + 鼠标滚轮）
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);

        if ctx.input(|i| i.modifiers.ctrl) && scroll != 0.0 {
            if let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos()) {

                let old_zoom = self.zoom;

                let zoom_speed = 0.0015; 
                let new_zoom = (old_zoom * (scroll * zoom_speed).exp()).clamp(0.05, 20.0);

                // 当前图片左上角
                let image_min = self.last_image_rect_min.unwrap_or(Pos2::ZERO);

                // 鼠标对应的图片坐标（缩放前）
                let image_pos = (mouse_pos - image_min) / old_zoom;

                // 更新 zoom
                self.zoom = new_zoom;

                // 重新计算 pan，使鼠标指向位置不变
                self.pan += image_pos * (old_zoom - new_zoom);
            }
        }

        // 撤销
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.rectangles.pop();
        }

        // 撤销
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_image(ctx);
        }

        self.toolbar(ctx);

        // 🟢 主画布
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                let image_size = texture.size_vec2() * self.zoom;

                // 分配可交互区域
                let (response, painter) =
                    ui.allocate_painter(image_size, Sense::click_and_drag());

                // 应用平移
                let image_rect = Rect::from_min_size(
                    response.rect.min + self.pan,
                    image_size,
                );
                self.last_image_rect_min = Some(image_rect.min);

                // 绘制图片
                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Ctrl + 左键拖动画布平移
                if self.current_tool == Tool::Select && ctx.input(|i| i.modifiers.ctrl) && response.dragged_by(egui::PointerButton::Primary) {
                    self.pan += response.drag_delta();
                }

                if let Some(pos) = response.hover_pos() {
                    let image_pos = self.screen_to_image(pos, image_rect);

                    ui.ctx().debug_painter().text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        format!("Image: {:.1}, {:.1}", image_pos.x, image_pos.y),
                        egui::FontId::monospace(12.0),
                        Color32::YELLOW,
                    );
                }

                if response.double_clicked() {
                    self.reset_view(response.rect);
                }

                // 只有在矩形模式才允许画
                if self.current_tool == Tool::Rectangle {
                    if response.drag_started() {
                        self.start_pos = response.interact_pointer_pos();
                    }

                    if response.drag_stopped() {
                        if let (Some(start), Some(end)) =
                            (self.start_pos, response.interact_pointer_pos())
                        {
                            let rect = Rect::from_two_pos(start, end);
                            self.rectangles.push(rect);
                        }
                        self.start_pos = None;
                    }
                }

                // 画已有矩形
                for rect in &self.rectangles {
                    painter.rect_stroke(
                        *rect,
                        0.0,
                        Stroke::new(self.line_width, self.current_color),
                        StrokeKind::Middle,
                    );
                }

                // 画当前拖动
                if self.current_tool == Tool::Rectangle {
                    if let (Some(start), Some(current)) =
                        (self.start_pos, response.interact_pointer_pos())
                    {
                        let rect = Rect::from_two_pos(start, current);
                        painter.rect_stroke(
                            rect,
                            0.0,
                            Stroke::new(self.line_width, egui::Color32::GREEN),
                            StrokeKind::Middle,
                        );
                    }
                }
            } else {
                ui.label("请在命令行传入图片路径");
            }
        });
    }
}
