use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    if !text.contains("ProfileCosmetics,") {
        text = text.replace("ThemeStudio,", "ThemeStudio,\n    ProfileCosmetics,");
    }
    if !text.contains("AppScreen::ProfileCosmetics => self.show_profile_cosmetics(ui),") {
        text = text.replace("AppScreen::ThemeStudio => self.show_theme_studio(ui),", "AppScreen::ThemeStudio => self.show_theme_studio(ui),\n                AppScreen::ProfileCosmetics => self.show_profile_cosmetics(ui),");
    }
    fs::write("src/main.rs", text).unwrap();
}
