use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    // Normalize newlines
    let text = text.replace("\r\n", "\n");
    let text = text.replace(
        "custom_themes: Vec::new(),\n        }\n    }",
        "custom_themes: Vec::new(),\n        };\n        app.apply_profile_theme();\n        app\n    }"
    );
    // Add missing show_theme_studio
    
    fs::write("src/main.rs", text).unwrap();
}
