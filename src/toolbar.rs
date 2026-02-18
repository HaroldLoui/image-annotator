use egui::{Button, Color32, Context, Frame, Image, Margin, TopBottomPanel, Ui, Vec2};

use crate::StrokeWidth;

const SELECT_ICON: &[u8] = include_bytes!("../assets/select.svg");
const RECT_ICON: &[u8] = include_bytes!("../assets/rect.svg");
const CIRCLE_ICON: &[u8] = include_bytes!("../assets/circle.svg");
const ARROW_ICON: &[u8] = include_bytes!("../assets/arrow.svg");
const LINE_ICON: &[u8] = include_bytes!("../assets/line.svg");
const PENCIL_ICON: &[u8] = include_bytes!("../assets/pencil.svg");
const NUMBER_ICON: &[u8] = include_bytes!("../assets/number.svg");
const EMOJI_ICON: &[u8] = include_bytes!("../assets/emoji.svg");
const TEXT_ICON: &[u8] = include_bytes!("../assets/text.svg");
const MOSAIC_ICON: &[u8] = include_bytes!("../assets/mosaic.svg");
const PIN_ICON: &[u8] = include_bytes!("../assets/pin.svg");
// const LINE_WIDTH_ICON: &[u8] = include_bytes!("../assets/lineWidth.svg");

const DOT_1_ICON: &[u8] = include_bytes!("../assets/dot1.svg");
const DOT_3_ICON: &[u8] = include_bytes!("../assets/dot3.svg");
const DOT_5_ICON: &[u8] = include_bytes!("../assets/dot5.svg");

/// 工具栏
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Tool {
    /// 选择
    Select,
    /// 框选
    Rectangle,
    /// 圈选
    Circle,
    /// 箭头
    Arrow,
    /// 画线
    Line,
    /// 画笔
    Pencil,
    /// 数字
    Number,
    /// Emoji
    Emoji,
    /// 文本
    Text,
    /// 马赛克
    Masaic,
    /// 铆钉
    Pin,
}

impl Tool {
    fn tool_info(&self) -> (&'static str, &'static [u8], &'static str) {
        match self {
            Tool::Select => ("bytes://select_icon.svg", SELECT_ICON, "Select"),
            Tool::Rectangle => ("bytes://rect_icon.svg", RECT_ICON, "Rect"),
            Tool::Arrow => ("bytes://arrow_icon.svg", ARROW_ICON, "Arrow"),
            Tool::Circle => ("bytes://circle_icon.svg", CIRCLE_ICON, "Circle"),
            Tool::Line => ("bytes://line_icon.svg", LINE_ICON, "Line"),
            Tool::Pencil => ("bytes://pencil_icon.svg", PENCIL_ICON, "Pencil"),
            Tool::Number => ("bytes://number_icon.svg", NUMBER_ICON, "Number"),
            Tool::Emoji => ("bytes://emoji_icon.svg", EMOJI_ICON, "Emoji"),
            Tool::Text => ("bytes://text_icon.svg", TEXT_ICON, "Text"),
            Tool::Masaic => ("bytes://mosaic_icon.svg", MOSAIC_ICON, "Mosaic"),
            Tool::Pin => ("bytes://pin_icon.svg", PIN_ICON, "Pin"),
        }
    }
}

impl crate::AnnotatorApp {
    const BUTTON_SIZE: Vec2 = egui::vec2(36.0, 36.0);

    /// 工具栏绘制函数
    pub fn toolbar(&mut self, ctx: &Context) {
        // 顶部工具栏
        TopBottomPanel::top("toolbar")
            .resizable(false)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 0))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(12.0, 0.0);

                            self.toolbar_button(ui, Tool::Select);
                            self.toolbar_button(ui, Tool::Rectangle);
                            self.toolbar_button(ui, Tool::Circle);
                            self.toolbar_button(ui, Tool::Arrow);
                            self.toolbar_button(ui, Tool::Line);
                            self.toolbar_button(ui, Tool::Pencil);
                            self.toolbar_button(ui, Tool::Number);
                            self.toolbar_button(ui, Tool::Emoji);
                            self.toolbar_button(ui, Tool::Text);
                            self.toolbar_button(ui, Tool::Masaic);
                            ui.separator();

                            self.line_width_button(ui, StrokeWidth::ONE);
                            self.line_width_button(ui, StrokeWidth::THREE);
                            self.line_width_button(ui, StrokeWidth::FIVE);
                            ui.separator();

                            if self.color_picker.ui(ui) {
                                self.current_color = self.color_picker.color();
                            }

                            // 右侧：设置和操作按钮
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("⚙").clicked() {}
                                    if ui.button("❓").clicked() {}
                                },
                            );
                        });
                    });
            });
    }

    // 工具栏图标按钮辅助函数
    fn toolbar_button(&mut self, ui: &mut egui::Ui, tool: Tool) {
        let selected = self.current_tool == tool;
        let (svg_uri, svg_data, tooltip) = tool.tool_info();
        let image = Image::from_bytes(svg_uri, svg_data);
        let button = Button::image(image)
            .fill(Color32::WHITE)
            .min_size(Self::BUTTON_SIZE)
            .frame(selected);
        if ui.add(button).on_hover_text(tooltip).clicked() {
            self.current_tool = tool;
        }
    }

    // 线宽选择按钮辅助函数
    fn line_width_button(&mut self, ui: &mut Ui, lw: StrokeWidth) {
        let data = match lw {
            StrokeWidth::ONE => ("bytes://dot1_icon.svg", DOT_1_ICON),
            StrokeWidth::THREE => ("bytes://dot3_icon.svg", DOT_3_ICON),
            StrokeWidth::FIVE => ("bytes://dot5_icon.svg", DOT_5_ICON),
            StrokeWidth::Custom(_) => unreachable!(),
        };
        let image = Image::from_bytes(data.0, data.1);
        let btn = Button::image(image)
            .fill(if self.stroke_width == lw {
                Color32::from_rgb(60, 60, 80)
            } else {
                Color32::from_rgb(40, 40, 50)
            })
            .min_size(Self::BUTTON_SIZE);

        if ui.add(btn).clicked() {
            self.stroke_width = lw;
        }
    }
}
