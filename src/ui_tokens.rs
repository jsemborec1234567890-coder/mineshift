use eframe::egui;

/// Renders a modern styled card frame with rounded corners and subtle stroke.
pub fn card_frame(card_bg: egui::Color32, border_color: egui::Color32, corner_radius: f32, padding: f32) -> egui::Frame {
    egui::Frame::new()
        .fill(card_bg)
        .corner_radius(corner_radius)
        .stroke(egui::Stroke::new(1.0, border_color))
        .inner_margin(padding)
}

/// Renders a horizontal header with a prominent back button, title, and optional badge.
#[allow(dead_code)]
pub fn screen_header(
    ui: &mut egui::Ui,
    back_label: &str,
    title: &str,
    right_badge: Option<(&str, egui::Color32, egui::Color32)>, // (text, bg, text_color)
    text_color: egui::Color32,
    border_color: egui::Color32,
    card_bg: egui::Color32,
) -> bool {
    let mut back_clicked = false;
    ui.horizontal(|ui| {
        let back_btn = egui::Button::new(
            egui::RichText::new(format!("← {}", back_label))
                .size(16.0)
                .color(text_color)
        )
        .fill(card_bg)
        .stroke(egui::Stroke::new(1.0, border_color))
        .corner_radius(8.0)
        .min_size(egui::vec2(90.0, 36.0));

        if ui.add(back_btn).clicked() {
            back_clicked = true;
        }

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(title)
                .size(26.0)
                .strong()
                .color(text_color)
        );

        if let Some((badge_text, badge_bg, badge_fg)) = right_badge {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::new()
                    .fill(badge_bg)
                    .corner_radius(16.0)
                    .inner_margin(egui::vec2(14.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(badge_text).size(14.0).strong().color(badge_fg));
                    });
            });
        }
    });
    back_clicked
}

/// Renders a sidebar tab button that fills when active.
#[allow(dead_code)]
pub fn sidebar_tab_button(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    accent: egui::Color32,
    card_bg: egui::Color32,
    text_primary: egui::Color32,
    _text_muted: egui::Color32,
) -> bool {
    let (bg_color, fg_color) = if is_active {
        (accent, egui::Color32::WHITE)
    } else {
        (card_bg, text_primary)
    };

    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(16.0)
            .strong()
            .color(fg_color)
    )
    .fill(bg_color)
    .corner_radius(10.0)
    .min_size(egui::vec2(ui.available_width(), 44.0));

    ui.add(btn).clicked()
}

/// Renders a modern slider row with label on the left and percentage/value on the right.
pub fn modern_slider_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    display_value: &str,
    accent: egui::Color32,
    text_color: egui::Color32,
    _muted_color: egui::Color32,
) -> bool {
    let mut changed = false;
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).size(16.0).color(text_color));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(display_value).size(16.0).strong().color(accent));
            });
        });
        ui.add_space(4.0);
        let slider = egui::Slider::new(value, range)
            .show_value(false);
        if ui.add_sized([ui.available_width(), 20.0], slider).changed() {
            changed = true;
        }
    });
    changed
}
