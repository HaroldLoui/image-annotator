#![allow(unused)]

use eframe::{App, egui};
use egui::{
    Color32, ColorImage, Image, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    TextureHandle, Vec2,
    epaint::{EllipseShape, PathShape, PathStroke},
};
use image::{DynamicImage, GenericImageView};

mod color_picker;
mod drawable;
mod operators;
mod toolbar;

use color_picker::ColorPickerButton;
use drawable::Draw;
use operators::{Operator, ToolType};
use toolbar::Tool;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Annotator",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(AnnotatorApp::new(cc)))
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrokeWidth {
    ONE,
    THREE,
    FIVE,
    Custom(f32),
}

impl Into<f32> for StrokeWidth {
    fn into(self) -> f32 {
        match self {
            StrokeWidth::ONE => 1f32,
            StrokeWidth::THREE => 3f32,
            StrokeWidth::FIVE => 5f32,
            StrokeWidth::Custom(x) => x,
        }
    }
}

struct AnnotatorApp {
    texture: Option<TextureHandle>,
    image_size: Vec2,
    image_path: Option<String>,
    current_tool: Tool,
    current_color: Color32,
    stroke_width: StrokeWidth,
    start_pos: Option<Pos2>,
    operators: Vec<Operator>,
    color_picker: ColorPickerButton,
    display_scale: f32,
    zoom: f32,
    pan: Vec2,
    last_image_rect: Option<Rect>,
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
            stroke_width: StrokeWidth::THREE,
            start_pos: None,
            operators: Vec::new(),
            color_picker: ColorPickerButton::new("ColorPicker", Color32::WHITE),
            display_scale,
            zoom: 1.0,
            pan: Vec2::ZERO,
            last_image_rect: None,
        }
    }

    fn save_image(&self, ctx: &egui::Context) {
        if let Some(path) = &self.image_path {
            let mut img = image::open(path)
                .expect("Failed to reopen image")
                .to_rgba8();

            for op in &self.operators {
                op.draw_on_image(self, &mut img);
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

    fn screen_to_image(&self, pos: Pos2, image_rect: Rect) -> Pos2 {
        (pos - image_rect.min).to_pos2() / self.zoom
    }

    fn reset_view(&mut self, available_rect: Rect) {
        if let Some(texture) = &self.texture {
            let image_size = texture.size_vec2();
            let panel_size = available_rect.size();

            let scale_x = panel_size.x / image_size.x;
            let scale_y = panel_size.y / image_size.y;

            self.zoom = scale_x.min(scale_y);

            let new_size = image_size * self.zoom;

            self.pan = (panel_size - new_size) / 2.0;
        }
    }
}

impl AnnotatorApp {
    fn drag_event(&mut self, response: &Response) {
        match self.current_tool {
            Tool::Select => {}
            Tool::Rectangle | Tool::Circle | Tool::Line => {
                if response.drag_started() {
                    self.start_pos = response.interact_pointer_pos();
                }

                if response.drag_stopped() {
                    if let (Some(start), Some(end)) =
                        (self.start_pos, response.interact_pointer_pos())
                    {
                        let op = self.get_operator(start, end).unwrap();
                        self.operators.push(op);
                    }
                    self.start_pos = None;
                }
            }
            Tool::Arrow => todo!(),
            Tool::Pencil => todo!(),
            Tool::Number => todo!(),
            Tool::Emoji => todo!(),
            Tool::Text => todo!(),
            Tool::Masaic => todo!(),
        }
    }

    fn drag_event_process(&self, response: &Response, painter: &Painter) {
        match self.current_tool {
            Tool::Select => {}
            Tool::Rectangle | Tool::Circle | Tool::Line => {
                if let (Some(start), Some(current)) =
                    (self.start_pos, response.interact_pointer_pos())
                {
                    let op = self.get_operator(start, current).unwrap();
                    op.draw(self, painter);
                }
            }
            Tool::Arrow => todo!(),
            Tool::Pencil => todo!(),
            Tool::Number => todo!(),
            Tool::Emoji => todo!(),
            Tool::Text => todo!(),
            Tool::Masaic => todo!(),
        }
    }

    fn get_operator(&self, start_pos: Pos2, end_pos: Pos2) -> Option<Operator> {
        // 获取当前 image_rect（需要存下来）
        let image_rect_min = self.last_image_rect.map_or(Pos2::ZERO, |r| r.min);
        let image_rect = Rect::from_min_size(image_rect_min, self.image_size * self.zoom);

        // 转换为图片坐标
        let start_img = self.screen_to_image(start_pos, image_rect);
        let end_img = self.screen_to_image(end_pos, image_rect);

        let width = self.stroke_width;
        let color = self.current_color;
        match self.current_tool {
            Tool::Select => None,
            Tool::Rectangle => {
                let rect = Rect::from_two_pos(start_img, end_img);
                Some(Operator::new(ToolType::Rect(rect), width, color))
            }
            Tool::Circle => {
                let radius = Vec2::new(
                    (end_img.x - start_img.x).abs() / 2.0,
                    (end_img.y - start_img.y).abs() / 2.0,
                );
                let center = Pos2::new(
                    (start_img.x + end_img.x) / 2.0,
                    (start_img.y + end_img.y) / 2.0,
                );
                let e = EllipseShape {
                    center,
                    radius,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(width, color),
                };
                Some(Operator::new(ToolType::Ellipse(e), width, color))
            }
            Tool::Arrow => {
                // TODO:
                PathShape {
                    points: vec![],
                    closed: true,
                    fill: color,
                    stroke: PathStroke::new(width, color),
                };
                None
            }
            Tool::Line => {
                Some(Operator::new(ToolType::Line(start_img, end_img), width, color))
            },
            Tool::Pencil => todo!(),
            Tool::Number => todo!(),
            Tool::Emoji => todo!(),
            Tool::Text => todo!(),
            Tool::Masaic => todo!(),
        }
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
    let color_image = ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], &rgba);
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
                let image_min = self.last_image_rect.map_or(Pos2::ZERO, |r| r.min);

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
            self.operators.pop();
        }

        // 保存
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_image(ctx);
        }

        self.toolbar(ctx);

        // 主画布
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                let image_size = texture.size_vec2() * self.zoom;

                // 分配可交互区域
                let (response, painter) = ui.allocate_painter(image_size, Sense::click_and_drag());

                // 应用平移
                let image_rect = Rect::from_min_size(response.rect.min + self.pan, image_size);
                self.last_image_rect = Some(image_rect);

                // 绘制图片
                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Ctrl + 左键拖动画布平移
                if self.current_tool == Tool::Select
                    && ctx.input(|i| i.modifiers.ctrl)
                    && response.dragged_by(egui::PointerButton::Primary)
                {
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

                // 根据工具进行拖拽绘制
                self.drag_event(&response);

                // 画已有标注
                for op in &self.operators {
                    op.draw(self, &painter);
                }

                // 画拖动过程
                self.drag_event_process(&response, &painter);
            } else {
                ui.label("请在命令行传入图片路径");
            }
        });
    }
}
