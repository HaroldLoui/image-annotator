use std::sync::Arc;

use ab_glyph::{Font, PxScale, ScaleFont};
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};

const DEFAULT_FONT_NAMES: &[&str] = &[
    "HarmonyOS Sans SC",
    "Noto Sans CJK SC Regular",
    "Microsoft YaHei UI",
    "DejaVu Sans",
    "Arial Unicode MS",
];

pub fn try_load_font_data_from_system() -> Option<(&'static [u8], String)> {
    for &family_name in DEFAULT_FONT_NAMES {
        let source = SystemSource::new();
        let handle = source
            .select_best_match(
                &[FamilyName::Title(family_name.to_string())],
                &Properties::new(),
            )
            .ok()?;
    
        let font = handle.load().ok()?;
        let data = font.copy_font_data()?;
        let data = Box::leak((*data).clone().into_boxed_slice());
        return Some((data, family_name.to_owned()));
    }
    None
}

pub fn init_egui_fonts(cc: &eframe::CreationContext<'_>, font_data: Option<(&'static [u8], String)>) {
    if let Some((data, font_name)) = font_data {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            font_name.clone(),
            Arc::new(egui::FontData::from_static(data)),
        );
        
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, font_name.clone());
    
        fonts.families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, font_name.clone());
        
        cc.egui_ctx.set_fonts(fonts);
    }
}

pub fn measure_text<F: Font>(font: &F, scale: PxScale, text: &str) -> (f32, f32) {
    let scaled = font.as_scaled(scale);
    let width: f32 = text.chars()
        .map(|c| scaled.h_advance(font.glyph_id(c)))
        .sum();
    let height = scaled.ascent() - scaled.descent();
    (width, height)
}