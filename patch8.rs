use std::fs;

fn main() {
    let mut text = fs::read_to_string("src/main.rs").unwrap();
    let show_theme_studio = r#"
    fn show_theme_studio(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = AppScreen::MainMenu;
                if let Some(ref prof) = self.active_profile {
                    if let Some(ref db) = self.db {
                        let _ = db.set_profile_theme(prof.id, &self.theme.id);
                    }
                }
            }
            ui.label(egui::RichText::new("ULTIMATE THEME STUDIO").size(24.0).strong().color(self.theme.colors.text_primary()));
        });
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.0);
                ui.heading("Presets");
                let presets = vec![
                    crate::theme::Theme::soft_circuit(),
                    crate::theme::Theme::obsidian(),
                    crate::theme::Theme::aurora(),
                    crate::theme::Theme::paper(),
                    crate::theme::Theme::retro(),
                    crate::theme::Theme::neon_arcade(),
                ];
                for preset in presets {
                    if ui.selectable_label(self.theme.id == preset.id, &preset.name).clicked() {
                        self.theme = preset;
                    }
                }
                ui.separator();
                ui.heading("Custom Themes");
                if let Some(ref prof) = self.active_profile {
                    if let Some(ref db) = self.db {
                        if let Ok(customs) = db.get_custom_themes(prof.id) {
                            for (id, name, data) in customs {
                                if ui.selectable_label(self.theme.id == id, &name).clicked() {
                                    if let Ok(mut imported) = serde_json::from_str::<crate::theme::Theme>(&data) {
                                        imported.is_custom = true;
                                        self.theme = imported;
                                    }
                                }
                            }
                        }
                    }
                }
                if ui.button("Create New").clicked() {
                    let mut new_theme = self.theme.clone();
                    new_theme.is_custom = true;
                    new_theme.id = format!("custom_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                    new_theme.name = format!("Custom Theme");
                    self.theme = new_theme;
                }
                if ui.button("Export").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        let data = serde_json::to_string_pretty(&self.theme).unwrap();
                        let _ = std::fs::write(path, data);
                    }
                }
                if ui.button("Import").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Ok(data) = std::fs::read_to_string(path) {
                            if let Ok(mut imported) = serde_json::from_str::<crate::theme::Theme>(&data) {
                                imported.is_custom = true;
                                imported.id = format!("custom_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                                self.theme = imported;
                            }
                        }
                    }
                }
                if self.theme.is_custom {
                    if ui.button("Save to Profile").clicked() {
                        if let Some(ref prof) = self.active_profile {
                            if let Some(ref db) = self.db {
                                let data = serde_json::to_string(&self.theme).unwrap();
                                let _ = db.save_custom_theme(prof.id, &self.theme.id, &self.theme.name, &data);
                            }
                        }
                    }
                }
            });
            
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(300.0);
                if self.theme.is_custom {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.theme.name);
                    });
                }
                
                ui.collapsing("Colors", |ui| {
                    ui.label("Background"); color_picker(ui, &mut self.theme.colors.bg);
                    ui.label("Menu Background"); color_picker(ui, &mut self.theme.colors.menu_bg);
                    ui.label("Card Background"); color_picker(ui, &mut self.theme.colors.card_bg);
                    ui.label("Border"); color_picker(ui, &mut self.theme.colors.border);
                    ui.label("Covered Cell"); color_picker(ui, &mut self.theme.colors.covered);
                    ui.label("Revealed Cell"); color_picker(ui, &mut self.theme.colors.revealed);
                    ui.label("Text Primary"); color_picker(ui, &mut self.theme.colors.text_primary);
                    ui.label("Text Muted"); color_picker(ui, &mut self.theme.colors.text_muted);
                    ui.label("Accent"); color_picker(ui, &mut self.theme.colors.accent);
                    ui.label("Danger"); color_picker(ui, &mut self.theme.colors.danger);
                    ui.label("Success"); color_picker(ui, &mut self.theme.colors.success);
                    ui.label("Button Background"); color_picker(ui, &mut self.theme.colors.button_bg);
                });
                
                ui.collapsing("Geometry", |ui| {
                    ui.add(egui::Slider::new(&mut self.theme.geometry.cell_spacing, 0.0..=10.0).text("Cell Spacing"));
                    ui.add(egui::Slider::new(&mut self.theme.geometry.square_radius, 0.0..=20.0).text("Square Radius"));
                    ui.add(egui::Slider::new(&mut self.theme.geometry.shadow_depth, 0.0..=10.0).text("Shadow Depth"));
                    ui.add(egui::Slider::new(&mut self.theme.geometry.border_thickness, 0.0..=5.0).text("Border Thickness"));
                });
                
                ui.collapsing("Numbers", |ui| {
                    for i in 0..8 {
                        if i < self.theme.colors.number_colors.len() {
                            ui.label(format!("Number {}", i + 1));
                            color_picker(ui, &mut self.theme.colors.number_colors[i]);
                        }
                    }
                });
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.heading("Live Preview");
                ui.label("Changes apply instantly.");
                
                let (rect, _response) = ui.allocate_exact_size(egui::vec2(200.0, 300.0), egui::Sense::hover());
                let center = rect.min + egui::vec2(100.0, 100.0);
                
                let cell_size = 30.0;
                let fill_color = self.theme.colors.covered();
                let shadow_depth = self.theme.geometry.shadow_depth;
                
                let square_rect = egui::Rect::from_center_size(center, egui::vec2(cell_size, cell_size));
                let shadow_rect = square_rect.translate(egui::vec2(0.0, 2.0));
                ui.painter().rect_filled(shadow_rect, self.theme.geometry.square_radius, egui::Color32::from_black_alpha((shadow_depth * 10.0) as u8));
                ui.painter().rect_filled(square_rect, self.theme.geometry.square_radius, fill_color);
                if self.theme.geometry.border_thickness > 0.0 {
                    ui.painter().rect_stroke(square_rect, self.theme.geometry.square_radius, egui::Stroke::new(self.theme.geometry.border_thickness, self.theme.colors.border()), egui::StrokeKind::Inside);
                }
                
                let hex_center = center + egui::vec2(0.0, 100.0);
                let mut points = vec![];
                let mut shadow_points = vec![];
                for i in 0..6 {
                    let angle = (i as f32 * 60.0 - 30.0).to_radians();
                    let p = hex_center + egui::vec2(angle.cos() * cell_size * 0.95, angle.sin() * cell_size * 0.95);
                    points.push(p);
                    shadow_points.push(p + egui::vec2(0.0, 2.0));
                }
                ui.painter().add(egui::Shape::convex_polygon(shadow_points, egui::Color32::from_black_alpha((shadow_depth * 10.0) as u8), egui::Stroke::NONE));
                ui.painter().add(egui::Shape::convex_polygon(points, fill_color, if self.theme.geometry.border_thickness > 0.0 { egui::Stroke::new(self.theme.geometry.border_thickness, self.theme.colors.border()) } else { egui::Stroke::NONE }));
            });
        });
    }
"#;
    let color_picker = r#"
fn color_picker(ui: &mut eframe::egui::Ui, color: &mut (u8, u8, u8)) {
    let mut c = [color.0, color.1, color.2];
    ui.color_edit_button_srgb(&mut c);
    *color = (c[0], c[1], c[2]);
}
"#;

    if !text.contains("fn show_theme_studio") {
        let text_no_cr = text.replace("\r\n", "\n");
        text = text_no_cr.replace("}\n}\n", &format!("{}\n}}\n{}", show_theme_studio, color_picker));
    }
    fs::write("src/main.rs", text).unwrap();
}
