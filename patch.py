import os
import re

file_path = "src/main.rs"
with open(file_path, "r", encoding="utf-8") as f:
    text = f.read()

# Add theme module
if "pub mod theme;" not in text:
    text = text.replace("pub mod db;", "pub mod db;\npub mod theme;")

# Add AppScreen::ThemeStudio
if "ThemeStudio" not in text:
    text = text.replace("Achievements,", "Achievements,\n    ThemeStudio,")

# Add theme to App struct
if "theme: theme::Theme," not in text:
    text = text.replace("generation_rx: Option<Receiver<Option<minecore::board::Board>>>,", "generation_rx: Option<Receiver<Option<minecore::board::Board>>>,\n    theme: theme::Theme,\n    custom_themes: Vec<theme::Theme>,")

# Add theme initialization to App::new
if "theme: theme::Theme::soft_circuit()" not in text:
    text = text.replace("generation_rx: None,", "generation_rx: None,\n            theme: theme::Theme::soft_circuit(),\n            custom_themes: Vec::new(),")

# Replace colors
text = text.replace("egui::Color32::from_rgb(0xEE, 0xE2, 0xFF)", "self.theme.colors.bg()")
text = text.replace("egui::Color32::from_rgb(0xFF, 0xFA, 0xFA)", "self.theme.colors.card_bg()")
text = text.replace("egui::Color32::from_rgb(0x33, 0x33, 0x33)", "self.theme.colors.text_primary()")
text = text.replace("egui::Color32::from_rgb(0x77, 0x77, 0x77)", "self.theme.colors.text_muted()")
text = text.replace("egui::Color32::from_rgb(0xFF, 0xA0, 0x7A)", "self.theme.colors.accent()")
text = text.replace("egui::Color32::from_rgb(0xDD, 0xDD, 0xDD)", "self.theme.colors.border()")
text = text.replace("egui::Color32::from_rgb(0xCC, 0xCC, 0xCC)", "self.theme.colors.covered()")
text = text.replace("egui::Color32::from_rgb(0xEE, 0xEE, 0xEE)", "self.theme.colors.revealed()")
text = text.replace("egui::Color32::from_rgb(0xFF, 0x6B, 0x6B)", "self.theme.colors.danger()")
text = text.replace("egui::Color32::from_rgb(0x4C, 0xAF, 0x50)", "self.theme.colors.success()")
text = text.replace("egui::Color32::from_rgb(0xF0, 0xF0, 0xF0)", "self.theme.colors.button_bg()")
text = text.replace("egui::Color32::from_rgb(0x9E, 0x9E, 0x9E)", "self.theme.colors.hover()")

# Also some local variable color declarations are in main menu and others
text = text.replace("let bg_color = egui::Color32::from_rgb(0xEE, 0xE2, 0xFF);", "let bg_color = self.theme.colors.bg();")
text = text.replace("let card_color = egui::Color32::from_rgb(0xFF, 0xFA, 0xFA);", "let card_color = self.theme.colors.card_bg();")
text = text.replace("let text_color = egui::Color32::from_rgb(0x33, 0x33, 0x33);", "let text_color = self.theme.colors.text_primary();")
text = text.replace("let accent_color = egui::Color32::from_rgb(0xFF, 0xA0, 0x7A);", "let accent_color = self.theme.colors.accent();")
text = text.replace("let muted_color = egui::Color32::from_rgb(0x77, 0x77, 0x77);", "let muted_color = self.theme.colors.text_muted();")
text = text.replace("let secondary_fill = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);", "let secondary_fill = self.theme.colors.button_bg();")
text = text.replace("let border_color = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);", "let border_color = self.theme.colors.border();")
text = text.replace("let danger_color = egui::Color32::from_rgb(0xFF, 0x6B, 0x6B);", "let danger_color = self.theme.colors.danger();")

# Geometry replacements
text = text.replace("let base_spacing = 2.0;", "let base_spacing = self.theme.geometry.cell_spacing;")
text = text.replace("ui.painter().rect_filled(shadow_rect, 6.0, egui::Color32::from_black_alpha(20));", "ui.painter().rect_filled(shadow_rect, self.theme.geometry.square_radius, egui::Color32::from_black_alpha((self.theme.geometry.shadow_depth * 10.0) as u8));")
text = text.replace("ui.painter().rect_filled(cell_rect, 6.0, fill_color);", "ui.painter().rect_filled(cell_rect, self.theme.geometry.square_radius, fill_color); if self.theme.geometry.border_thickness > 0.0 { ui.painter().rect_stroke(cell_rect, self.theme.geometry.square_radius, egui::Stroke::new(self.theme.geometry.border_thickness, self.theme.colors.border()), egui::StrokeKind::Inside); }")
text = text.replace("ui.painter().add(egui::Shape::convex_polygon(shadow_points, egui::Color32::from_black_alpha(20), egui::Stroke::NONE));", "ui.painter().add(egui::Shape::convex_polygon(shadow_points, egui::Color32::from_black_alpha((self.theme.geometry.shadow_depth * 10.0) as u8), egui::Stroke::NONE));")
text = text.replace("ui.painter().add(egui::Shape::convex_polygon(points, fill_color, egui::Stroke::NONE));", "ui.painter().add(egui::Shape::convex_polygon(points, fill_color, if self.theme.geometry.border_thickness > 0.0 { egui::Stroke::new(self.theme.geometry.border_thickness, self.theme.colors.border()) } else { egui::Stroke::NONE }));")

# Match screen
text = text.replace("AppScreen::Achievements => self.show_achievements(ui),", "AppScreen::Achievements => self.show_achievements(ui),\n                AppScreen::ThemeStudio => self.show_theme_studio(ui),")

# Apply theme helper
apply_theme = """
    fn apply_profile_theme(&mut self) {
        if let Some(ref prof) = self.active_profile {
            if let Some(theme) = crate::theme::Theme::get_preset(&prof.selected_theme_id) {
                self.theme = theme;
            } else if let Some(ref db) = self.db {
                if let Ok(customs) = db.get_custom_themes(prof.id) {
                    if let Some((_, _, data)) = customs.into_iter().find(|(id, _, _)| id == &prof.selected_theme_id) {
                        if let Ok(mut t) = serde_json::from_str::<crate::theme::Theme>(&data) {
                            t.is_custom = true;
                            self.theme = t;
                        }
                    }
                }
            }
        }
    }
"""
if "fn apply_profile_theme" not in text:
    text = text.replace("impl MineShiftApp {", "impl MineShiftApp {" + apply_theme)

# Call apply_profile_theme at end of new()
if "app.apply_profile_theme()" not in text:
    text = text.replace("        Self {", "        let mut app = Self {")
    text = text.replace("            custom_themes: Vec::new(),\n        }\n    }", "            custom_themes: Vec::new(),\n        };\n        app.apply_profile_theme();\n        app\n    }")

with open(file_path, "w", encoding="utf-8") as f:
    f.write(text)
