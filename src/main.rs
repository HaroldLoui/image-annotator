// #![allow(unused)]

use std::sync::mpsc::{self, Receiver};

use eframe::{App, egui};
use egui::{
    Color32, ColorImage, Painter, PointerButton, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2, epaint::{EllipseShape, PathShape, PathStroke}
};

mod color_picker;
mod drawable;
mod operators;
mod toolbar;

use color_picker::ColorPickerButton;
use drawable::Draw;
use image::RgbaImage;
use operators::{Operator, ToolType};
use toolbar::{Tool, arrow_points};

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
    // 显示
    texture: Option<TextureHandle>,
    last_image_rect: Option<Rect>,
    display_scale: f32,
    zoom: f32,
    pan: Vec2,
    // 图片相关
    image_size: Vec2,
    image_path: Option<String>,
    original_image: Option<RgbaImage>,
    image_receiver: Option<Receiver<(RgbaImage, Vec2, f32)>>,
    // 工具相关
    current_tool: Tool,
    color_picker: ColorPickerButton,
    current_color: Color32,
    stroke_width: StrokeWidth,
    start_pos: Option<Pos2>,
    tracks: Vec<Option<Pos2>>,
    operators: Vec<Operator>,
}

impl AnnotatorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 从命令行读取图片路径
        let args: Vec<String> = std::env::args().collect();
        let mut image_path = None;
        let mut image_receiver = None;

        if args.len() > 1 {
            let (tx, rx) = mpsc::channel();
            let ctx = cc.egui_ctx.clone();
            let path = args[1].clone();

            std::thread::spawn(move || {
                let img = image::open(&path)
                    .expect("Failed to reopen image")
                    .to_rgba8();
                let mut image_size = Vec2::ZERO;
                let (_, scale) = scale_color_image(&img, &mut image_size); // 复用现有逻辑
                tx.send((img, image_size, scale)).unwrap();
                ctx.request_repaint(); // 通知 UI 刷新
            });
            image_path = Some(args[1].clone());
            image_receiver = Some(rx);
        }

        Self {
            texture: None,
            last_image_rect: None,
            display_scale: 1.0,
            zoom: 1.0,
            pan: Vec2::ZERO,
            image_size: Vec2::ZERO,
            image_path,
            original_image: None,
            image_receiver,
            current_tool: Tool::Rectangle,
            color_picker: ColorPickerButton::new("ColorPicker", Color32::WHITE),
            current_color: Color32::WHITE,
            stroke_width: StrokeWidth::THREE,
            start_pos: None,
            tracks: Vec::new(),
            operators: Vec::new(),
        }
    }

    fn save_image(&self, ctx: &egui::Context) {
        if let (Some(path), Some(img)) = (&self.image_path, &self.original_image) {
            let mut img = img.clone();

            for op in &self.operators {
                op.draw_on_image(&mut img);
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
        (pos - image_rect.min).to_pos2() / (self.zoom * self.display_scale)
    }

    fn image_to_screen(&self, p: Pos2) -> Pos2 {
        let image_rect_min = self.last_image_rect.map_or(Pos2::ZERO, |r| r.min);
        image_rect_min + p.to_vec2() * self.zoom
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
    fn drag_event(&mut self, ui: &Ui, response: &Response) {
        match self.current_tool {
            Tool::Select => {}
            Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Arrow => {
                if response.drag_started_by(PointerButton::Primary) {
                    self.start_pos = response.interact_pointer_pos();
                }

                if response.drag_stopped_by(PointerButton::Primary) {
                    if let (Some(start), Some(end)) =
                        (self.start_pos, response.interact_pointer_pos())
                    {
                        let op = self.get_operator(start, end).unwrap();
                        self.operators.push(op);
                    }
                    self.start_pos = None;
                }
            }
            Tool::Pencil => {
                // FIXME: 
                if response.drag_started_by(PointerButton::Primary) {
                    if let Some(origin) = ui.input(|i| i.pointer.press_origin()) {
                        self.tracks.push(Some(origin));
                    }
                }
                if response.dragged_by(PointerButton::Primary) {
                    self.tracks.push(response.interact_pointer_pos());
                }
                if response.drag_stopped_by(PointerButton::Primary) {
                    self.tracks.push(response.interact_pointer_pos());
                    if let Some(op) = self.get_operator(Pos2::ZERO, Pos2::ZERO) {
                        self.operators.push(op);
                    }
                    self.tracks.clear();
                }
            }
            Tool::Number => todo!(),
            Tool::Emoji => todo!(),
            Tool::Text => todo!(),
            Tool::Masaic => todo!(),
            Tool::Pin => todo!(),
        }
    }

    fn drag_event_process(&self, response: &Response, painter: &Painter) {
        match self.current_tool {
            Tool::Select => {}
            Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Arrow => {
                if let (Some(start), Some(current)) =
                    (self.start_pos, response.interact_pointer_pos())
                {
                    let op = self.get_operator(start, current).unwrap();
                    op.draw(self, painter);
                }
            }
            Tool::Pencil => {
                if let Some(op) = self.get_operator(Pos2::ZERO, Pos2::ZERO) {
                    op.draw(self, painter);
                }
            },
            Tool::Number => todo!(),
            Tool::Emoji => todo!(),
            Tool::Text => todo!(),
            Tool::Masaic => todo!(),
            Tool::Pin => todo!(),
        }
    }

    fn get_operator(&self, start_pos: Pos2, end_pos: Pos2) -> Option<Operator> {
        // 获取当前 image_rect（需要存下来）
        let image_rect_min = self.last_image_rect.map_or(Pos2::ZERO, |r| r.min);
        let image_rect = Rect::from_min_size(image_rect_min, self.image_size * self.zoom);

        // 转换为图片坐标
        let start = self.screen_to_image(start_pos, image_rect);
        let end = self.screen_to_image(end_pos, image_rect);

        let width = self.stroke_width;
        let color = self.current_color;
        match self.current_tool {
            Tool::Select => None,
            Tool::Rectangle => {
                let rect = Rect::from_two_pos(start, end);
                Some(Operator::new(ToolType::Rect(rect), width, color))
            }
            Tool::Circle => {
                let radius = Vec2::new(
                    (end.x - start.x).abs() / 2.0, 
                    (end.y - start.y).abs() / 2.0
                );
                let center = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
                let e = EllipseShape {
                    center,
                    radius,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(width, color),
                };
                Some(Operator::new(ToolType::Ellipse(e), width, color))
            }
            Tool::Arrow => {
                let ps = PathShape {
                    points: arrow_points(start, end, width),
                    closed: true,
                    fill: color,
                    stroke: PathStroke::new(width, color),
                };
                Some(Operator::new(ToolType::Arrow(ps), width, color))
            }
            Tool::Line => Some(Operator::new(ToolType::Line(start, end), width, color)),
            Tool::Pencil => {
                if self.tracks.is_empty() {
                    None
                } else {
                    let points: Vec<Pos2> = self.tracks
                        .iter()
                        .filter(|opt| opt.is_some())
                        .map(|opt| {
                            let p = opt.unwrap();
                            self.screen_to_image(p, image_rect)
                        })
                        .collect();
                    if points.is_empty() {
                        return None;
                    }
                    Some(Operator::new(ToolType::Pencil(points), width, color))
                }
            },
            Tool::Number => todo!(),
            Tool::Emoji => todo!(),
            Tool::Text => todo!(),
            Tool::Masaic => todo!(),
            Tool::Pin => todo!(),
        }
    }
}

fn scale_color_image(img: &RgbaImage, image_size: &mut Vec2) -> (ColorImage, f32) {
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
            img,
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
        // 检查图片是否加载完成
        if let Some(rx) = &self.image_receiver {
            if let Ok((img, image_size, scale)) = rx.try_recv() {
                let mut size = image_size;
                let (color_image, _) = scale_color_image(&img, &mut size);
                self.texture = Some(ctx.load_texture(
                    "loaded_image", color_image, Default::default()
                ));
                self.image_size = image_size;
                self.display_scale = scale;
                self.original_image = Some(img);
                self.image_receiver = None; // 清掉 receiver
            }
        }

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
                // painter 占满整个面板，不随 zoom 变化
                let available = ui.available_rect_before_wrap();
                let (response, painter) =
                    ui.allocate_painter(available.size(), Sense::click_and_drag());

                // 图片的实际渲染区域
                let image_size = texture.size_vec2() * self.zoom;
                let image_rect = Rect::from_min_size(available.min + self.pan, image_size);
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
                self.drag_event(ui, &response);

                // 画已有标注
                for op in &self.operators {
                    op.draw(self, &painter);
                }

                // 画拖动过程
                self.drag_event_process(&response, &painter);
            } else {
                // 显示 loading
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Loading image...");
                });
            }
        });
    }
}
