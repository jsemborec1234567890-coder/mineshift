# Settings Crash Diagnostic Report

## Environment
- Executable: 	arget/debug/mineshift.exe
- Trigger: Clicking the 'SETTINGS' button on the Main Menu.

## Root Cause Evidence
Inspection of src/main.rs near line 250 reveals the following logic for the SETTINGS button:

`ust
if ui.add(egui::Button::new(egui::RichText::new("SETTINGS").size(18.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(400.0, 48.0))).clicked() {
    // Future settings screen, or just Exit for now
    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
}
`

## Conclusion
The application is not experiencing a Rust panic, memory violation, or threading deadlock. Instead, the SETTINGS button is hardcoded to emit egui::ViewportCommand::Close. The window cleanly terminates, which appears from the outside as an immediate crash to desktop.

Additionally, the EXIT button requested in the main menu design was omitted, causing SETTINGS to act as the de facto exit button.

## Required Fix
1. Create a true AppScreen::Settings.
2. Map SETTINGS to navigate to self.screen = AppScreen::Settings.
3. Add a proper EXIT button that issues ViewportCommand::Close.
4. Populate the new Settings screen with actual configurable state (Cursor, Animation Speed, Reduced Motion) mapped directly to the self.theme settings to preserve persistence.
