use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    let old = "            ui.add_space(12.0);\n            if ui.add(egui::Button::new(egui::RichText::new(\"Exit\")";
    let new = "            ui.add_space(12.0);\n            if ui.add(egui::Button::new(egui::RichText::new(\"Theme Studio\").size(20.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(300.0, 44.0))).clicked() {\n                self.screen = AppScreen::ThemeStudio;\n            }\n            ui.add_space(12.0);\n            if ui.add(egui::Button::new(egui::RichText::new(\"Exit\")";
    text = text.replace(old, new);
    fs::write("src/main.rs", text).unwrap();
}
