use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    if !text.contains("AppScreen::ThemeStudio => self.show_theme_studio(ui),") {
        text = text.replace("AppScreen::GeneratingNoGuess => self.show_generating(ui),", "AppScreen::GeneratingNoGuess => self.show_generating(ui),\n                AppScreen::ThemeStudio => self.show_theme_studio(ui),");
    }
    
    // Fix rect_stroke StrokeKind
    text = text.replace(
        "ui.painter().rect_stroke(square_rect, self.theme.geometry.square_radius, egui::Stroke::new(self.theme.geometry.border_thickness, self.theme.colors.border()));",
        "ui.painter().rect_stroke(square_rect, self.theme.geometry.square_radius, egui::Stroke::new(self.theme.geometry.border_thickness, self.theme.colors.border()), egui::StrokeKind::Inside);"
    );
    
    fs::write("src/main.rs", text).unwrap();
}
