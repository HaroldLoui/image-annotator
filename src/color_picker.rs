use egui::{Color32, Response, StrokeKind, Ui, Vec2};

/// 颜色选择器按钮组件
pub struct ColorPickerButton {
    /// 当前选中的颜色
    current_color: Color32,
    /// 临时颜色（用户正在选择但未确认的颜色）
    temp_color: Color32,
    /// 是否显示选择器窗口
    show_picker: bool,
    /// 是否展开自定义颜色选择器
    show_custom: bool,
    /// 唯一ID（用于窗口）
    id: String,
    /// 刚刚打开标志（用于防止立即关闭）
    just_opened: bool,
}

impl ColorPickerButton {
    /// 创建新的颜色选择器按钮
    pub fn new(id: impl Into<String>, initial_color: Color32) -> Self {
        Self {
            current_color: initial_color,
            temp_color: initial_color,
            show_picker: false,
            show_custom: false,
            id: id.into(),
            just_opened: false,
        }
    }

    /// 获取当前颜色
    pub fn color(&self) -> Color32 {
        self.current_color
    }

    /// 设置颜色
    pub fn set_color(&mut self, color: Color32) {
        self.current_color = color;
        self.temp_color = color;
    }

    /// 显示颜色选择器按钮
    /// 返回：(Response, Option<Color32>)
    /// - Response: 按钮的响应
    /// - Option<Color32>: 如果用户点击了确认，返回新选择的颜色
    pub fn show(&mut self, ui: &mut Ui) -> (Response, Option<Color32>) {
        let button_response = self.draw_button(ui);
        
        if button_response.clicked() {
            self.show_picker = true;
            self.just_opened = true;
            self.temp_color = self.current_color;
        }

        let mut selected_color = None;

        if self.show_picker {
            let window_id = egui::Id::new(&self.id);
            
            let window_response = egui::Window::new("Color Picker")
                .id(window_id)
                .collapsible(false)
                .resizable(false)
                .default_width(280.0)
                .show(ui.ctx(), |ui| {
                    // 预设颜色
                    ui.heading("Preset Colors");
                    ui.add_space(5.0);
                    
                    self.draw_preset_colors(ui);
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    // 自定义颜色
                    ui.horizontal(|ui| {
                        ui.heading("Custom Color");
                        if ui.button(if self.show_custom { "▼" } else { "▶" }).clicked() {
                            self.show_custom = !self.show_custom;
                        }
                    });
                    
                    if self.show_custom {
                        ui.add_space(5.0);
                        ui.color_edit_button_srgba(&mut self.temp_color);
                    }
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    // 预览和确认按钮
                    let mut close_window = false;
                    let mut confirm = false;
                    
                    ui.horizontal(|ui| {
                        ui.label("Current:");
                        self.draw_color_preview(ui, self.temp_color);
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Confirm").clicked() {
                                confirm = true;
                                close_window = true;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                close_window = true;
                            }
                        });
                    });
                    
                    (close_window, confirm)
                });
            
            // 处理窗口返回值
            if let Some(inner_response) = window_response {
                if let Some((close_window, confirm)) = inner_response.inner {
                    if confirm {
                        self.current_color = self.temp_color;
                        selected_color = Some(self.current_color);
                    }
                    if close_window {
                        self.show_picker = false;
                        if !confirm {
                            self.temp_color = self.current_color;
                        }
                    }
                }
                
                // 检测窗口是否被关闭（点击外部区域）
                // 但忽略刚打开时的点击
                if inner_response.response.clicked_elsewhere() && !self.just_opened {
                    self.show_picker = false;
                    self.temp_color = self.current_color;
                }
            }
            
            // 重置 just_opened 标志
            if self.just_opened {
                self.just_opened = false;
            }
        }

        (button_response, selected_color)
    }

    /// 绘制颜色选择器按钮
    fn draw_button(&self, ui: &mut Ui) -> Response {
        let button_size = Vec2::new(30.0, 30.0);
        let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            
            // 绘制按钮背景
            ui.painter().rect(
                rect,
                3.0,
                visuals.bg_fill,
                visuals.bg_stroke,
                StrokeKind::Middle
            );

            // 绘制颜色预览方块（左侧）
            let color_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(5.0, 5.0),
                Vec2::new(20.0, 20.0),
            );
            
            ui.painter().rect_filled(color_rect, 2.0, self.current_color);
            ui.painter().rect_stroke(
                color_rect,
                2.0,
                egui::Stroke::new(1.0, Color32::GRAY),
                StrokeKind::Middle
            );

            // 绘制文本（右侧）
            // let text_pos = color_rect.right_center() + egui::vec2(8.0, 0.0);
            // let (r, g, b, _) = self.current_color.to_tuple();
            // let text = format!("RGB\n{},{},{}", r, g, b);
            
            // ui.painter().text(
            //     text_pos,
            //     egui::Align2::LEFT_CENTER,
            //     text,
            //     egui::FontId::proportional(10.0),
            //     visuals.text_color(),
            // );
        }

        response
    }

    /// 绘制预设颜色网格
    fn draw_preset_colors(&mut self, ui: &mut Ui) {
        const PRESET_COLORS: &[Color32] = &[
            Color32::BLACK,
            Color32::WHITE,
            Color32::from_rgb(255, 0, 0),     // 红
            Color32::from_rgb(0, 255, 0),     // 绿
            Color32::from_rgb(0, 0, 255),     // 蓝
            Color32::from_rgb(255, 255, 0),   // 黄
            Color32::from_rgb(255, 165, 0),   // 橙
            Color32::from_rgb(128, 0, 128),   // 紫
            Color32::from_rgb(255, 192, 203), // 粉
            Color32::GRAY,
            Color32::from_rgb(139, 69, 19),   // 棕
            Color32::from_rgb(0, 255, 255),   // 青
            Color32::from_rgb(255, 0, 255),   // 洋红
            Color32::LIGHT_GRAY,
            Color32::DARK_GRAY,
        ];

        egui::Grid::new(format!("{}_preset_grid", self.id))
            .spacing([5.0, 5.0])
            .show(ui, |ui| {
                for (i, &color) in PRESET_COLORS.iter().enumerate() {
                    if self.draw_color_tile(ui, color) {
                        self.temp_color = color;
                    }

                    if (i + 1) % 5 == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    /// 绘制单个颜色方块
    fn draw_color_tile(&self, ui: &mut Ui, color: Color32) -> bool {
        let size = Vec2::new(40.0, 40.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let inner_rect = rect.shrink(2.0);
            
            // 绘制颜色方块
            ui.painter().rect_filled(inner_rect, 2.0, color);

            // 绘制边框（选中状态）
            let is_selected = self.temp_color == color;
            let stroke = if is_selected {
                egui::Stroke::new(3.0, Color32::from_rgb(100, 150, 255))
            } else if response.hovered() {
                egui::Stroke::new(2.0, Color32::LIGHT_GRAY)
            } else {
                egui::Stroke::new(1.0, Color32::GRAY)
            };

            ui.painter().rect_stroke(inner_rect, 2.0, stroke, StrokeKind::Middle);
        }

        response.clicked()
    }

    /// 绘制颜色预览
    fn draw_color_preview(&self, ui: &mut Ui, color: Color32) {
        let size = Vec2::new(60.0, 20.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

        ui.painter().rect_filled(rect, 2.0, color);
        ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0, Color32::GRAY), StrokeKind::Middle);

        let (r, g, b, _) = color.to_tuple();
        ui.label(format!("({},{},{})", r, g, b));
    }
}

// 简化的 API
impl ColorPickerButton {
    /// 简化版本：只显示按钮，返回是否有颜色被选中
    pub fn ui(&mut self, ui: &mut Ui) -> bool {
        let (_, changed) = self.show(ui);
        changed.is_some()
    }

    /// 带标签的版本
    pub fn ui_with_label(&mut self, ui: &mut Ui, label: &str) -> Option<Color32> {
        ui.horizontal(|ui| {
            ui.label(label);
            let (_, color) = self.show(ui);
            color
        }).inner
    }
}