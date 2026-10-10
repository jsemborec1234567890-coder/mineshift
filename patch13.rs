use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    let old_menu = "            ui.add_space(12.0);\n            if ui.add(egui::Button::new(egui::RichText::new(\"Exit\")";
    let new_menu = "            ui.add_space(12.0);\n            if ui.add(egui::Button::new(egui::RichText::new(\"Theme Studio\").size(20.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(300.0, 44.0))).clicked() {\n                self.screen = AppScreen::ThemeStudio;\n            }\n            ui.add_space(12.0);\n            if ui.add(egui::Button::new(egui::RichText::new(\"Exit\")";
    text = text.replace(old_menu, new_menu);
    
    // Add cosmetics button to show_profiles
    // We'll put it right after the Back button in the top horizontal layout
    let old_prof = "if self.active_profile.is_some() {\n                if ui.button(egui::RichText::new(\"â—€ Back\").size(20.0)).clicked() {\n                    self.screen = AppScreen::MainMenu;\n                }\n                ui.add_space(16.0);\n            }";
    let new_prof = "if self.active_profile.is_some() {\n                if ui.button(egui::RichText::new(\"â—€ Back\").size(20.0)).clicked() {\n                    self.screen = AppScreen::MainMenu;\n                }\n                ui.add_space(16.0);\n                if ui.button(egui::RichText::new(\"âœ¨ Customize Cosmetics\").size(20.0)).clicked() {\n                    self.screen = AppScreen::ProfileCosmetics;\n                }\n                ui.add_space(16.0);\n            }";
    text = text.replace(old_prof, new_prof);
    
    fs::write("src/main.rs", text).unwrap();
}
