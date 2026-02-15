use egui::{Button, Color32, Context, Frame, Image, Margin, TopBottomPanel};

const SELECT_ICON: &str = r#"
    <svg viewBox="0 0 24 24">
    <path fill="white" d="M3 2l7 20 2-8 8-2z"/>
    </svg>
    "#;

const RECT_ICON: &str = r#"
    <svg viewBox="0 0 24 24">
    <rect x="4" y="4" width="16" height="16"
            stroke="white" fill="none" stroke-width="2"/>
    </svg>
    "#;

/// 工具栏
#[derive(PartialEq)]
pub enum Tool {
    Select,
    Rectangle,
}

impl crate::AnnotatorApp {
    pub fn toolbar(&mut self, ctx: &Context) {
        // 🔵 顶部工具栏
        TopBottomPanel::top("toolbar")
            .resizable(false)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 0))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(12.0, 0.0);

                            let size = egui::vec2(36.0, 36.0);

                            svg_tool_button(
                                ui,
                                size,
                                &mut self.current_tool,
                                Tool::Select,
                                "bytes://select_icon.svg",
                                SELECT_ICON,
                                "Select (V)",
                            );
                            svg_tool_button(
                                ui,
                                size,
                                &mut self.current_tool,
                                Tool::Rectangle,
                                "bytes://rect_icon.svg",
                                RECT_ICON,
                                "Rect (R)",
                            );
                        });
                    });
            });
    }
}

fn svg_tool_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    current: &mut Tool,
    tool: Tool,
    svg_uri: &'static str,
    svg_data: &'static str,
    tooltip: &str,
) {
    let selected = *current == tool;
    let image = Image::from_bytes(svg_uri, svg_data.as_bytes());
    let button = Button::image(image).min_size(size).frame(selected);
    if ui.add(button).on_hover_text(tooltip).clicked() {
        *current = tool;
    }
}
