// #![allow(unused)]

use std::sync::mpsc::{self, Receiver};

use eframe::{App, egui, wgpu};
use egui::{Color32, ColorImage, Pos2, Rect, Sense, TextureHandle, Vec2};

mod color_picker;
mod drawable;
mod operators;
mod toolbar;
mod utils;

use color_picker::ColorPickerButton;
use drawable::DrawImage;
use image::RgbaImage;
use operators::Operator;
use toolbar::Tool;

use crate::{operators::ToolType, toolbar::ToolInfo, utils::AppHelper};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                eframe::egui_wgpu::WgpuSetupCreateNew {
                    device_descriptor: std::sync::Arc::new(|_adapter| {
                        wgpu::DeviceDescriptor {
                            required_limits: wgpu::Limits {
                                max_texture_dimension_2d: 8192, // 调大
                                ..wgpu::Limits::default()
                            },
                            ..Default::default()
                        }
                    }),
                    ..Default::default()
                },
            ),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "Annotator",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(AnnotatorApp::new(cc)))
        }),
    )
}

#[derive(Default)]
struct AnnotatorApp {
    // 显示
    texture: Option<TextureHandle>,
    last_image_rect: Option<Rect>,
    zoom: f32,
    pan: Vec2,
    // 图片相关
    image_size: Vec2,
    original_image: Option<RgbaImage>,
    image_receiver: Option<Receiver<(RgbaImage, Vec2)>>,
    // 工具相关
    color_picker: ColorPickerButton,
    current_tool_info: ToolInfo,
    // 进行过的操作
    operators: Vec<Operator>,
}

impl AnnotatorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 从命令行读取图片路径
        let args: Vec<String> = std::env::args().collect();
        let mut image_receiver = None;

        if args.len() > 1 {
            let (tx, rx) = mpsc::channel();
            let ctx = cc.egui_ctx.clone();
            let path = args[1].clone();

            std::thread::spawn(move || {
                let img = image::open(&path)
                    .expect("Failed to reopen image")
                    .to_rgba8();
                let dim = img.dimensions();
                let image_size = Vec2::new(dim.0 as f32, dim.1 as f32);
                tx.send((img, image_size)).unwrap();
                ctx.request_repaint();
            });
            image_receiver = Some(rx);
        }

        Self {
            zoom: 1.0,
            image_receiver,
            color_picker: ColorPickerButton::new("ColorPicker", Color32::RED),
            current_tool_info: ToolInfo::new(Color32::RED),
            ..Default::default()
        }
    }

    fn save_image(&self, ctx: &egui::Context) {
        if let Some(img) = &self.original_image {
            let mut img = img.clone();

            for op in &self.operators {
                op.draw_on_image(&mut img);
            }

            img.save_with_format("output.png", image::ImageFormat::Png).expect("Failed to save image");
            println!("image saved!");

            // let _ = std::process::Command::new("wl-copy")
            //     .arg("--type")
            //     .arg("image/png")
            //     .arg(path)
            //     .spawn();

            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
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

impl App for AnnotatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查图片是否加载完成
        if let Some(rx) = &self.image_receiver {
            if let Ok((img, image_size)) = rx.try_recv() {
                let color_image = ColorImage::from_rgba_unmultiplied(
                    [image_size.x as usize, image_size.y as usize], 
                    &img.as_raw()
                );
                self.texture = Some(ctx.load_texture("loaded_image", color_image, Default::default()));
                self.image_size = image_size;
                self.original_image = Some(img);
                self.image_receiver = None;
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
            let opt = self.operators.pop();
            if opt.is_some_and(|op| matches!(op.tool, ToolType::Number(..))) {
                self.current_tool_info.number -= 1;
            }
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
                if self.current_tool_info.tool == Tool::Select
                    && ctx.input(|i| i.modifiers.ctrl)
                    && response.dragged_by(egui::PointerButton::Primary)
                {
                    self.pan += response.drag_delta();
                }

                let helper = AppHelper::from_app(self);
                if let Some(pos) = response.hover_pos() {
                    let image_pos = helper.screen_to_image(pos, None);

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
                if let Some(op) = self.current_tool_info.drag_event(&helper, ui, &response) {
                    self.operators.push(op);
                }

                // 画已有标注
                for op in &self.operators {
                    op.draw(&helper, &painter);
                }

                // 画拖动过程
                self.current_tool_info.drag_event_process(&helper, &painter, &response);
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
