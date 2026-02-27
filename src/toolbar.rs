use std::ops::Mul;

use egui::{
    Button, Color32, Context, Frame, Image, Margin, Painter, PointerButton, Pos2, Rect, Response,
    Stroke, TopBottomPanel, Ui, Vec2,
    epaint::{CircleShape, EllipseShape, PathShape, PathStroke},
};

use crate::{
    operators::{Operator, ToolType},
    utils::AppHelper,
};

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
const COPY_ICON: &[u8] = include_bytes!("../assets/copy.svg");
const SAVE_ICON: &[u8] = include_bytes!("../assets/save.svg");

const DOT_1_ICON: &[u8] = include_bytes!("../assets/dot1.svg");
const DOT_3_ICON: &[u8] = include_bytes!("../assets/dot3.svg");
const DOT_5_ICON: &[u8] = include_bytes!("../assets/dot5.svg");

/// 工具栏
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum Tool {
    /// 选择
    #[default]
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
    /// Pin
    Pin,
    /// 复制到剪贴板
    Copy,
    /// 保存到本地
    Save,
}

impl Tool {
    /// 工具栏图标信息
    fn tool_icon(&self) -> (&'static str, &'static [u8], &'static str) {
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
            Tool::Copy => ("bytes://copy_icon.svg", COPY_ICON, "Copy to Clipboard"),
            Tool::Save => ("bytes://save_icon.svg", SAVE_ICON, "Save"),
        }
    }
}

/// 绘制工具栏
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
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(12.0, 0.0);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
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

                                    self.toolbar_button(ui, Tool::Pin);
                                    self.toolbar_button(ui, Tool::Copy);
                                    self.toolbar_button(ui, Tool::Save);
                                });
                                ui.separator();
                                ui.horizontal_centered(|ui| {
                                    self.line_width_button(ui, StrokeWidth::ONE);
                                    self.line_width_button(ui, StrokeWidth::THREE);
                                    self.line_width_button(ui, StrokeWidth::FIVE);
                                    ui.separator();
                                    if self.color_picker.ui(ui) {
                                        self.current_tool_info.color = self.color_picker.color();
                                    }
                                });
                            });
                        });
                    });
            });
    }

    // 工具栏图标按钮辅助函数
    fn toolbar_button(&mut self, ui: &mut Ui, tool: Tool) {
        let selected = self.current_tool_info.tool == tool;
        let (svg_uri, svg_data, tooltip) = tool.tool_icon();
        let image = Image::from_bytes(svg_uri, svg_data);
        let button = Button::image(image)
            .fill(Color32::LIGHT_GRAY)
            .min_size(Self::BUTTON_SIZE)
            .frame(selected);
        if ui.add(button).on_hover_text(tooltip).clicked() {
            self.current_tool_info.tool = tool;
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
        let selected = self.current_tool_info.stroke_width == lw;
        let btn = Button::image(image)
            .fill(Color32::LIGHT_GRAY)
            .min_size(Self::BUTTON_SIZE)
            .frame(selected);

        if ui.add(btn).clicked() {
            self.current_tool_info.stroke_width = lw;
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum StrokeWidth {
    ONE,
    #[default]
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

impl Mul<f32> for StrokeWidth {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        let base: f32 = self.into();
        base * rhs
    }
}

impl Mul<StrokeWidth> for f32 {
    type Output = f32;

    fn mul(self, rhs: StrokeWidth) -> Self::Output {
        let base: f32 = rhs.into();
        self * base
    }
}
#[derive(Debug, Default, Clone)]
pub struct TextEditState {
    pub pos: Pos2,
    pub content: String,
}

/// 当前选择的工具信息及事件相关属性
#[derive(Debug, Default, Clone)]
pub struct ToolInfo {
    pub tool: Tool,
    pub stroke_width: StrokeWidth,
    pub color: Color32,
    pub start_pos: Option<Pos2>,
    pub end_pos: Option<Pos2>,
    pub tracks: Vec<Option<Pos2>>,
    pub number: u8,
    pub text_editing: Option<TextEditState>,
}

impl ToolInfo {
    pub fn new(color: Color32) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// 事件：拖动，点击..
    pub fn input_event(
        &mut self,
        helper: &AppHelper,
        ui: &mut Ui,
        response: &Response,
    ) -> Option<Operator> {
        match self.tool {
            Tool::Select => {}
            Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Arrow => {
                if response.drag_started_by(PointerButton::Primary) {
                    if let Some(origin) = ui.input(|i| i.pointer.press_origin()) {
                        self.start_pos = Some(origin);
                    }
                }

                if response.drag_stopped_by(PointerButton::Primary) {
                    self.end_pos = response.interact_pointer_pos();
                    if self.start_pos.is_some() && self.end_pos.is_some() {
                        let opt = self.get_operator(helper, self.end_pos);
                        self.start_pos = None;
                        self.end_pos = None;
                        return opt;
                    }
                }
            }
            Tool::Pencil => {
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
                    let opt = self.get_operator(helper, self.end_pos);
                    self.tracks.clear();
                    return opt;
                }
            }
            Tool::Number => {
                if response.clicked_by(PointerButton::Primary) {
                    if let Some(point) = response.interact_pointer_pos() {
                        self.start_pos = Some(point);
                        let opt = self.get_operator(helper, None);
                        self.start_pos = None;
                        self.number += 1;
                        return opt;
                    }
                }
            }
            Tool::Emoji => {}
            Tool::Text => {
                if self.text_editing.is_none() && response.clicked_by(PointerButton::Primary) {
                    if let Some(point) = response.interact_pointer_pos() {
                        let img_pos = helper.screen_to_image(point, None);
                        self.text_editing = Some(TextEditState {
                            pos: img_pos,
                            content: String::new(),
                        });
                    }
                }
            }
            Tool::Masaic => {}
            Tool::Pin => {}
            Tool::Copy => {}
            Tool::Save => {}
        }
        None
    }

    pub fn input_event_process(
        &mut self,
        helper: &AppHelper,
        painter: &Painter,
        response: &Response,
    ) {
        match self.tool {
            Tool::Select => {}
            Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Arrow => {
                if self.start_pos.is_some() {
                    if let Some(end) = response.interact_pointer_pos() {
                        let op = self.get_operator(helper, Some(end)).unwrap();
                        op.draw(helper, painter);
                    }
                }
            }
            Tool::Pencil => {
                if let Some(end) = response.interact_pointer_pos() {
                    if let Some(op) = self.get_operator(helper, Some(end)) {
                        op.draw(helper, painter);
                    }
                }
            }
            Tool::Number => {
                if self.start_pos.is_some() {
                    if let Some(op) = self.get_operator(helper, None) {
                        op.draw(helper, painter);
                    }
                }
            }
            Tool::Emoji => {}
            Tool::Text => {}
            Tool::Masaic => {}
            Tool::Pin => {}
            Tool::Copy => {}
            Tool::Save => {}
        }
    }

    pub fn get_operator(&self, helper: &AppHelper, end_pos: Option<Pos2>) -> Option<Operator> {
        // 获取当前 image_rect（需要存下来）
        let image_rect = Some(helper.get_image_rect());

        // 转换为图片坐标
        let start = helper.screen_to_image(self.start_pos.unwrap_or_default(), image_rect);
        let end = helper.screen_to_image(end_pos.unwrap_or_default(), image_rect);

        let width = self.stroke_width;
        let color = self.color;
        match self.tool {
            Tool::Select => None,
            Tool::Rectangle => {
                let rect = Rect::from_two_pos(start, end);
                Some(Operator::new(ToolType::Rect(rect), width, color, None))
            }
            Tool::Circle => {
                let radius =
                    Vec2::new((end.x - start.x).abs() / 2.0, (end.y - start.y).abs() / 2.0);
                let center = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
                let e = EllipseShape {
                    center,
                    radius,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(width, color),
                };
                Some(Operator::new(ToolType::Ellipse(e), width, color, None))
            }
            Tool::Arrow => {
                let ps = PathShape {
                    points: arrow_points(start, end, width),
                    closed: true,
                    fill: color,
                    stroke: PathStroke::new(width, color),
                };
                Some(Operator::new(
                    ToolType::Arrow(ps),
                    width,
                    color,
                    Some(color),
                ))
            }
            Tool::Line => Some(Operator::new(
                ToolType::Line(start, end),
                width,
                color,
                None,
            )),
            Tool::Pencil => {
                if self.tracks.is_empty() {
                    None
                } else {
                    let points: Vec<Pos2> = self
                        .tracks
                        .iter()
                        .filter(|opt| opt.is_some())
                        .map(|opt| {
                            let p = opt.unwrap();
                            helper.screen_to_image(p, image_rect)
                        })
                        .collect();
                    if points.is_empty() {
                        return None;
                    }
                    Some(Operator::new(ToolType::Pencil(points), width, color, None))
                }
            }
            Tool::Number => {
                let radius = 10.0 + width * 5.0;
                let shape = CircleShape {
                    center: start,
                    radius,
                    fill: color,
                    stroke: Stroke::new(1.0, Color32::BLACK),
                };
                Some(Operator::new(
                    ToolType::Number(shape, self.number),
                    width,
                    color,
                    Some(color),
                ))
            }
            Tool::Emoji => todo!(),
            Tool::Text => None, // 需要等输入完成后才创建 Operator
            Tool::Masaic => todo!(),
            Tool::Pin => todo!(),
            Tool::Copy => todo!(),
            Tool::Save => todo!(),
        }
    }
}

pub fn arrow_points(start: Pos2, end: Pos2, stroke: StrokeWidth) -> Vec<Pos2> {
    let dir = end - start;
    let len = dir.length();

    if len <= f32::EPSILON {
        return vec![];
    }
    let stroke: f32 = stroke.into();

    let v = dir / len;
    let n = Vec2::new(-v.y, v.x);

    let head_len = stroke * 8.0;
    let head_width = stroke * 5.0;
    let shaft_width = stroke;

    let head_base = end - v * head_len;

    let p0 = end;

    let p1 = head_base + n * (head_width * 0.5);
    let p6 = head_base - n * (head_width * 0.5);

    let p2 = head_base + n * (shaft_width * 0.5);
    let p5 = head_base - n * (shaft_width * 0.5);

    let p3 = start + n * (shaft_width * 0.5);
    let p4 = start - n * (shaft_width * 0.5);

    vec![p0, p1, p2, p3, p4, p5, p6]
}
