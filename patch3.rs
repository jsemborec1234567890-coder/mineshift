use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    if !text.contains("theme: crate::theme::Theme,") {
        text = text.replace("generation_rx: Option<std::sync::mpsc::Receiver<(Option<Board>, usize, usize)>>,", "generation_rx: Option<std::sync::mpsc::Receiver<(Option<Board>, usize, usize)>>,\n    theme: crate::theme::Theme,\n    custom_themes: Vec<crate::theme::Theme>,");
    }
    if !text.contains("theme: crate::theme::Theme::soft_circuit(),") {
        text = text.replace("generation_rx: None,", "generation_rx: None,\n            theme: crate::theme::Theme::soft_circuit(),\n            custom_themes: Vec::new(),");
    }
    fs::write("src/main.rs", text).unwrap();
}
