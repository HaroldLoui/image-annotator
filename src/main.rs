#![allow(unused)]

use eframe::{App, egui};
use egui::{Color32, ColorImage, Pos2, Rect, Stroke, StrokeKind, TextureHandle};
use image::GenericImageView;

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
    image_size: egui::Vec2,
    image_path: Option<String>,
    current_tool: Tool,
    current_color: Color32,
    line_width: LineWidth,
    start_pos: Option<Pos2>,
    rectangles: Vec<Rect>,
    color_picker: ColorPickerButton,
}

impl AnnotatorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 从命令行读取图片路径
        let args: Vec<String> = std::env::args().collect();
        let mut texture = None;
        let mut image_size = egui::Vec2::ZERO;
        let mut image_path = None;

        if args.len() > 1 {
            image_path = Some(args[1].clone());
            let img = image::open(&args[1]).expect("Failed to open image");
            let size = img.dimensions();
            image_size = egui::vec2(size.0 as f32, size.1 as f32);

            let rgba = img.to_rgba8();
            let color_image =
                ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], &rgba);

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

        let min_x = rect.min.x as u32;
        let min_y = rect.min.y as u32;
        let max_x = rect.max.x as u32;
        let max_y = rect.max.y as u32;

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
}

impl App for AnnotatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                let response =
                    ui.add(egui::Image::new(texture).sense(egui::Sense::click_and_drag()));
                let painter = ui.painter_at(response.rect);

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
