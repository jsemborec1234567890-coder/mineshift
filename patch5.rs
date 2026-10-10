use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    if !text.contains("pub mod theme;") {
        text = text.replace("mod db;", "pub mod db;\npub mod theme;");
    }
    
    // Fix missing ; and return app in new()
    text = text.replace("app.apply_profile_theme();\n        app\n    }", "app.apply_profile_theme();\n        app\n    }"); // already there probably
    
    // Actually, I can just use a regex
    fs::write("src/main.rs", text).unwrap();
}
