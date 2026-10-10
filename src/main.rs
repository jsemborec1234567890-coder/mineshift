mod board;
mod db;

use board::{Board, GameStatus, CellState};
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "MINE//SHIFT",
        options,
        Box::new(|_cc| Ok(Box::new(MineShiftApp::new()))),
    )
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    egui::Color32::from_rgb(
        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
    )
}

#[derive(PartialEq)]
enum AppScreen {
    MainMenu,
    Gameplay,
    Profiles,
    Statistics,
    Achievements,
}

struct MineShiftApp {
    screen: AppScreen,
    board: Board,
    db: Option<db::Database>,
    active_profile: Option<db::Profile>,
    profiles: Vec<db::Profile>,
    new_profile_name: String,
    rename_profile_id: Option<i64>,
    rename_buffer: String,
}

impl MineShiftApp {
    fn new() -> Self {
        let db = db::Database::new().ok();
        let mut profiles = Vec::new();
        let mut active_profile = None;
        
        if let Some(ref d) = db {
            if let Ok(mut profs) = d.get_profiles() {
                if !profs.is_empty() {
                    active_profile = Some(profs[0].clone());
                }
                profiles = profs;
            }
        }
        
        let screen = if active_profile.is_some() { AppScreen::MainMenu } else { AppScreen::Profiles };

        Self {
            screen,
            board: Board::new_square(9, 9, 10),
            db,
            active_profile,
            profiles,
            new_profile_name: String::new(),
            rename_profile_id: None,
            rename_buffer: String::new(),
        }
    }
    
    fn show_main_menu(&mut self, ui: &mut egui::Ui) {
        let time = ui.input(|i| i.time);
        ui.ctx().request_repaint();
        
        let painter = ui.painter();
        let rect = ui.max_rect();
        let center = rect.center();
        for i in 0..6 {
            let radius = 150.0 + i as f32 * 100.0;
            let angle = time as f32 * (0.05 + i as f32 * 0.02);
            
            let mut points = vec![];
            for j in 0..6 {
                let a = angle + (j as f32 * 60.0).to_radians();
                points.push(center + egui::vec2(a.cos() * radius, a.sin() * radius));
            }
            
            painter.add(egui::Shape::closed_line(
                points,
                egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(121, 101, 190, 40))
            ));
        }

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.15);
            
            ui.label(
                egui::RichText::new("MINE//SHIFT")
                    .size(80.0)
                    .strong()
                    .extra_letter_spacing(4.0)
                    .color(egui::Color32::from_rgb(0x30, 0x30, 0x38))
            );
            
            ui.add_space(60.0);
            
            ui.style_mut().spacing.button_padding = egui::vec2(32.0, 16.0);
            
            let play_square = egui::Button::new(
                egui::RichText::new("PLAY SQUARE").size(24.0).color(egui::Color32::from_rgb(0xFF, 0xD6, 0xA8))
            ).fill(egui::Color32::from_rgb(0x79, 0x65, 0xBE)).min_size(egui::vec2(300.0, 54.0));
            
            if ui.add(play_square).clicked() {
                self.board = Board::new_square(16, 16, 40);
                self.screen = AppScreen::Gameplay;
            }
            
            ui.add_space(16.0);
            
            let play_hex = egui::Button::new(
                egui::RichText::new("PLAY HEXAGON").size(24.0).color(egui::Color32::from_rgb(0xFF, 0xD6, 0xA8))
            ).fill(egui::Color32::from_rgb(0x79, 0x65, 0xBE)).min_size(egui::vec2(300.0, 54.0));
            
            if ui.add(play_hex).clicked() {
                self.board = Board::new_hex(8, 40);
                self.screen = AppScreen::Gameplay;
            }
            
            ui.add_space(32.0);
            
            let secondary_fill = egui::Color32::from_rgb(0xA1, 0xA0, 0xA8);
            let text_color = egui::Color32::from_rgb(0x30, 0x30, 0x38);
            
            ui.add_space(32.0);
            if ui.add(egui::Button::new(egui::RichText::new("Profiles").size(20.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(300.0, 44.0))).clicked() {
                if let Some(ref d) = self.db {
                    self.profiles = d.get_profiles().unwrap_or_default();
                }
                self.screen = AppScreen::Profiles;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("Statistics").size(20.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(300.0, 44.0))).clicked() {
                self.screen = AppScreen::Statistics;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("Achievements").size(20.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(300.0, 44.0))).clicked() {
                self.screen = AppScreen::Achievements;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("Exit").size(20.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(300.0, 44.0))).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        
        // Show Active Profile Info at Top Right
        if let Some(ref profile) = self.active_profile {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.add_space(16.0);
                egui::Frame::new().inner_margin(16.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("Level {}", profile.current_level)).size(20.0).strong().color(egui::Color32::from_rgb(0x79, 0x65, 0xBE)));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(&profile.display_name).size(20.0).color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(&profile.avatar_id).size(24.0)); // Avatar emoji
                    });
                });
            });
        }
    }

    fn show_gameplay_header(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::new()
            .fill(egui::Color32::from_rgb(0xFF, 0xFF, 0xFF))
            .inner_margin(16.0);
            
        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(16.0, 8.0);
                
                if ui.button(egui::RichText::new("◀ Menu").size(18.0)).clicked() {
                    self.screen = AppScreen::MainMenu;
                }
                
                ui.add_space(10.0);
                
                let mode_text = match self.board.geometry {
                    board::Geometry::Square => format!("SQUARE {}×{}", self.board.width, self.board.height),
                    board::Geometry::Hex => format!("HEXAGON R{}", self.board.radius),
                };
                ui.label(egui::RichText::new(mode_text).size(16.0).color(egui::Color32::from_rgb(0xA1, 0xA0, 0xA8)));
                
                ui.add_space(20.0);
                
                let elapsed = if let Some(end) = self.board.end_time {
                    if let Some(start) = self.board.start_time {
                        end.duration_since(start).as_secs()
                    } else { 0 }
                } else if let Some(start) = self.board.start_time {
                    std::time::Instant::now().duration_since(start).as_secs()
                } else { 0 };

                ui.label(egui::RichText::new(format!("⏱ {:03}", elapsed)).size(24.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
                
                ui.add_space(20.0);
                
                let mines_left = self.board.mine_count as isize - self.board.flags_placed as isize;
                ui.label(egui::RichText::new(format!("🚩 {}", mines_left)).size(24.0).strong().color(egui::Color32::from_rgb(0xE8, 0x5C, 0x60)));
                
                ui.add_space(20.0);
                
                let status_text = match self.board.status {
                    GameStatus::Playing => egui::RichText::new("PLAYING").color(egui::Color32::from_rgb(0xA1, 0xA0, 0xA8)),
                    GameStatus::Won => egui::RichText::new("VICTORY").color(egui::Color32::from_rgb(0x60, 0xCD, 0xB1)),
                    GameStatus::Lost => egui::RichText::new("DEFEAT").color(egui::Color32::from_rgb(0xE8, 0x5C, 0x60)),
                };
                ui.label(status_text.size(24.0).strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Restart").size(18.0)).clicked() {
                        self.board = match self.board.geometry {
                            board::Geometry::Square => Board::new_square(self.board.width, self.board.height, self.board.mine_count),
                            board::Geometry::Hex => Board::new_hex(self.board.radius, self.board.mine_count),
                        };
                    }
                });
            });
        });
    }

    fn show_gameplay(&mut self, ui: &mut egui::Ui) {
        self.show_gameplay_header(ui);
        ui.add_space(20.0);
            
            ui.vertical_centered(|ui| {
                let avail = ui.available_size();
                let base_cell_size = 32.0;
                let base_spacing = 2.0;
                
                let raw_w = match self.board.geometry {
                    board::Geometry::Square => (base_cell_size + base_spacing) * self.board.width as f32,
                    board::Geometry::Hex => (base_cell_size * 3.0_f32.sqrt() + base_spacing) * self.board.width as f32,
                };
                let raw_h = match self.board.geometry {
                    board::Geometry::Square => (base_cell_size + base_spacing) * self.board.height as f32,
                    board::Geometry::Hex => (base_cell_size * 1.5 + base_spacing) * self.board.height as f32,
                };
                
                let scale = (avail.x / raw_w).min(avail.y / raw_h).min(1.5);
                let cell_size = base_cell_size * scale;
                let spacing = base_spacing * scale;
                
                let board_w = raw_w * scale;
                let board_h = raw_h * scale;
                
                let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(board_w, board_h),
                        egui::Sense::click(),
                    );
                    
                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                    let mut clicked_cell = None;
                    let mut secondary_clicked_cell = None;
                    let mut middle_clicked_cell = None;
                    let mut hovered_cell = None;
                    
                    // We need to find the hovered cell manually for hex, and can do it for square too for consistency.
                    let mut closest_dist = f32::MAX;
                    
                    for y in 0..self.board.height {
                        for x in 0..self.board.width {
                            let cell = self.board.get_cell(x, y);
                            if cell.is_none() { continue; }
                            let cell = cell.unwrap();
                            if !cell.active { continue; }
                            
                            let center = match self.board.geometry {
                                board::Geometry::Square => {
                                    rect.min + egui::vec2(
                                        x as f32 * (cell_size + spacing) + cell_size / 2.0,
                                        y as f32 * (cell_size + spacing) + cell_size / 2.0
                                    )
                                }
                                board::Geometry::Hex => {
                                    let q = x as f32 - self.board.radius as f32;
                                    let r = y as f32 - self.board.radius as f32;
                                    let cx = cell_size * 3.0_f32.sqrt() * (q + r / 2.0);
                                    let cy = cell_size * 1.5 * r;
                                    rect.min + egui::vec2(board_w / 2.0 + cx, board_h / 2.0 + cy)
                                }
                            };
                            
                            if let Some(pos) = pointer_pos {
                                let dist = (pos - center).length();
                                if dist < cell_size / 2.0 && dist < closest_dist {
                                    closest_dist = dist;
                                    hovered_cell = Some((x, y));
                                    if response.clicked() { clicked_cell = Some((x, y)); }
                                    if response.secondary_clicked() { secondary_clicked_cell = Some((x, y)); }
                                    if response.middle_clicked() { middle_clicked_cell = Some((x, y)); }
                                }
                            }
                            
                            let is_hovered = hovered_cell == Some((x, y));
                            let hover_t = ui.ctx().animate_bool(egui::Id::new(("hover", x, y)), is_hovered);
                            let reveal_t = ui.ctx().animate_bool(egui::Id::new(("reveal", x, y)), cell.state == CellState::Revealed);
                            
                            let covered_base = egui::Color32::from_rgb(0xA1, 0xA0, 0xA8);
                            let covered_hov = egui::Color32::from_rgb(0xB0, 0xAF, 0xB7);
                            let covered_col = lerp_color(covered_base, covered_hov, hover_t);
                            
                            let revealed_base = if cell.is_mine {
                                egui::Color32::from_rgb(0xE8, 0x5C, 0x60)
                            } else {
                                egui::Color32::from_rgb(0xFF, 0xD6, 0xA8)
                            };
                            
                            let fill_color = lerp_color(covered_col, revealed_base, reveal_t);
                            
                            let charcoal = egui::Color32::from_rgb(0x30, 0x30, 0x38);
                            
                            match self.board.geometry {
                                board::Geometry::Square => {
                                    let cell_rect = egui::Rect::from_center_size(center, egui::vec2(cell_size, cell_size));
                                    let shadow_rect = cell_rect.translate(egui::vec2(0.0, 2.0));
                                    ui.painter().rect_filled(shadow_rect, 6.0, egui::Color32::from_black_alpha(20));
                                    ui.painter().rect_filled(cell_rect, 6.0, fill_color);
                                }
                                board::Geometry::Hex => {
                                    let mut points = vec![];
                                    let mut shadow_points = vec![];
                                    for i in 0..6 {
                                        let angle_deg = 60.0 * i as f32 - 30.0;
                                        let angle_rad = angle_deg.to_radians();
                                        let p = center + egui::vec2(angle_rad.cos() * cell_size * 0.95, angle_rad.sin() * cell_size * 0.95);
                                        points.push(p);
                                        shadow_points.push(p + egui::vec2(0.0, 2.0));
                                    }
                                    ui.painter().add(egui::Shape::convex_polygon(shadow_points, egui::Color32::from_black_alpha(20), egui::Stroke::NONE));
                                    ui.painter().add(egui::Shape::convex_polygon(points, fill_color, egui::Stroke::NONE));
                                }
                            }
                            
                            match cell.state {
                                CellState::Flagged => {
                                    let base_y = center.y + cell_size * 0.2;
                                    let tip_y = center.y - cell_size * 0.25;
                                    let flag_right = center.x + cell_size * 0.2;
                                    ui.painter().line_segment([egui::pos2(center.x, base_y), egui::pos2(center.x, tip_y)], egui::Stroke::new(2.0, charcoal));
                                    ui.painter().add(egui::Shape::convex_polygon(
                                        vec![egui::pos2(center.x, tip_y), egui::pos2(flag_right, tip_y + cell_size * 0.1), egui::pos2(center.x, tip_y + cell_size * 0.2)],
                                        egui::Color32::from_rgb(0xE8, 0x5C, 0x60),
                                        egui::Stroke::NONE,
                                    ));
                                }
                                CellState::Revealed => {
                                    if cell.is_mine {
                                        ui.painter().circle_filled(center, cell_size * 0.25, charcoal);
                                        for i in 0..8 {
                                            let a = (i as f32 * 45.0).to_radians();
                                            let r = cell_size * 0.35;
                                            ui.painter().line_segment([center, center + egui::vec2(a.cos() * r, a.sin() * r)], egui::Stroke::new(2.0, charcoal));
                                        }
                                        ui.painter().circle_filled(center + egui::vec2(-cell_size * 0.08, -cell_size * 0.08), cell_size * 0.08, egui::Color32::WHITE);
                                    } else if cell.adjacent_mines > 0 {
                                        let text_color = match cell.adjacent_mines {
                                            1 => egui::Color32::from_rgb(0x50, 0x80, 0xC0), // Muted Blue
                                            2 => egui::Color32::from_rgb(0x40, 0xA0, 0x80), // Teal
                                            3 => egui::Color32::from_rgb(0xC0, 0x40, 0x40), // Red
                                            4 => egui::Color32::from_rgb(0x79, 0x65, 0xBE), // Violet
                                            _ => charcoal,
                                        };
                                        ui.painter().text(center, egui::Align2::CENTER_CENTER, cell.adjacent_mines.to_string(), egui::FontId::proportional(cell_size * 0.7), text_color);
                                    }
                                }
                                CellState::Covered => {}
                            }
                        }
                    }
                    
                    let old_status = self.board.status;
                    if let Some((x, y)) = clicked_cell { self.board.reveal(x, y); }
                    if let Some((x, y)) = secondary_clicked_cell { self.board.toggle_flag(x, y); }
                    if let Some((x, y)) = middle_clicked_cell { self.board.chord(x, y); }
                    
                    if old_status == GameStatus::Playing && self.board.status != GameStatus::Playing {
                        if let (Some(db), Some(profile)) = (&self.db, &self.active_profile) {
                            let duration_ms = if let (Some(s), Some(e)) = (self.board.start_time, self.board.end_time) {
                                e.duration_since(s).as_millis() as i64
                            } else { 0 };
                            let started_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 - (duration_ms / 1000);
                            
                            let session = db::GameSession {
                                profile_id: profile.id,
                                mode: "Classic".to_string(),
                                geometry: match self.board.geometry { board::Geometry::Square => "Square".to_string(), board::Geometry::Hex => "Hex".to_string() },
                                board_width: self.board.width as i64,
                                board_height: self.board.height as i64,
                                hex_radius: self.board.radius as i64,
                                mine_count: self.board.mine_count as i64,
                                started_at,
                                duration_ms,
                                result: match self.board.status { GameStatus::Won => "Won".to_string(), _ => "Lost".to_string() }
                            };
                            let _ = db.save_session(&session);
                            let _ = db.add_stat(profile.id, "total_playtime_s", duration_ms / 1000);
                            
                            if self.board.status == GameStatus::Won {
                                let _ = db.add_xp(profile.id, 50);
                                if self.board.geometry == board::Geometry::Square {
                                    let _ = db.set_best_time(profile.id, "best_time_square", duration_ms / 1000);
                                    let _ = db.unlock_achievement(profile.id, "first_win");
                                    if self.board.mine_count >= 99 {
                                        let _ = db.unlock_achievement(profile.id, "expert_clear");
                                    }
                                } else {
                                    let _ = db.set_best_time(profile.id, "best_time_hex", duration_ms / 1000);
                                    let _ = db.unlock_achievement(profile.id, "hex_master");
                                }
                            }
                            self.profiles = db.get_profiles().unwrap_or_default();
                            if let Some(p) = self.profiles.iter().find(|p| p.id == profile.id) {
                                self.active_profile = Some(p.clone());
                            }
                        }
                    }
                });

        if matches!(self.board.status, GameStatus::Playing) && self.board.start_time.is_some() {
            ui.ctx().request_repaint();
        }
    }
    fn show_profiles(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if self.active_profile.is_some() {
                if ui.button(egui::RichText::new("◀ Back").size(20.0)).clicked() {
                    self.screen = AppScreen::MainMenu;
                }
                ui.add_space(16.0);
            }
            ui.label(egui::RichText::new("PLAYER PROFILES").size(32.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
        });
        ui.add_space(16.0);
        
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let mut action: Option<(&str, db::Profile)> = None;
            
            for profile in &self.profiles {
                let is_active = self.active_profile.as_ref().map_or(false, |p| p.id == profile.id);
                let bg_color = if is_active { egui::Color32::from_rgb(0xFF, 0xD6, 0xA8) } else { egui::Color32::from_rgb(0xFF, 0xFF, 0xFF) };
                
                egui::Frame::new().fill(bg_color).inner_margin(16.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&profile.avatar_id).size(32.0));
                        ui.vertical(|ui| {
                            if Some(profile.id) == self.rename_profile_id {
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut self.rename_buffer);
                                    if ui.button("Save").clicked() {
                                        action = Some(("rename_save", profile.clone()));
                                    }
                                    if ui.button("Cancel").clicked() {
                                        action = Some(("rename_cancel", profile.clone()));
                                    }
                                });
                            } else {
                                ui.label(egui::RichText::new(&profile.display_name).size(24.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
                            }
                            ui.label(egui::RichText::new(format!("Level {}", profile.current_level)).size(16.0).color(egui::Color32::from_rgb(0x79, 0x65, 0xBE)));
                            
                            let mut req = 100;
                            let mut lvl = 1;
                            let mut prev_req = 0;
                            while lvl < profile.current_level {
                                prev_req = req;
                                lvl += 1;
                                req += lvl * 100;
                            }
                            let progress = (profile.total_xp as f32 - prev_req as f32) / (req as f32 - prev_req as f32).max(1.0);
                            
                            let pb = egui::ProgressBar::new(progress).text(format!("{} / {} XP", profile.total_xp, req)).fill(egui::Color32::from_rgb(0x79, 0x65, 0xBE));
                            ui.add_sized([150.0, 16.0], pb);
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !is_active {
                                if ui.button(egui::RichText::new("Delete").size(16.0).color(egui::Color32::from_rgb(0xE8, 0x5C, 0x60))).clicked() {
                                    action = Some(("delete", profile.clone()));
                                }
                                ui.add_space(8.0);
                                if ui.button(egui::RichText::new("Rename").size(16.0)).clicked() {
                                    action = Some(("rename", profile.clone()));
                                }
                                ui.add_space(8.0);
                                if ui.button(egui::RichText::new("Select").size(16.0)).clicked() {
                                    action = Some(("select", profile.clone()));
                                }
                            } else {
                                if ui.button(egui::RichText::new("Rename").size(16.0)).clicked() {
                                    action = Some(("rename", profile.clone()));
                                }
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("Active").strong());
                            }
                        });
                    });
                });
                ui.add_space(10.0);
            }
            
            if let Some((cmd, profile)) = action {
                if cmd == "delete" {
                    if let Some(ref db) = self.db {
                        let _ = db.delete_profile(profile.id);
                        self.profiles = db.get_profiles().unwrap_or_default();
                    }
                } else if cmd == "select" {
                    self.active_profile = Some(profile.clone());
                    if let Some(ref db) = self.db {
                        let _ = db.update_profile_last_played(profile.id);
                    }
                    self.screen = AppScreen::MainMenu;
                } else if cmd == "rename" {
                    self.rename_profile_id = Some(profile.id);
                    self.rename_buffer = profile.display_name.clone();
                } else if cmd == "rename_save" {
                    if let Some(ref db) = self.db {
                        let _ = db.rename_profile(profile.id, &self.rename_buffer);
                        self.profiles = db.get_profiles().unwrap_or_default();
                        if let Some(p) = self.profiles.iter().find(|p| p.id == profile.id) {
                            if self.active_profile.as_ref().map_or(false, |a| a.id == p.id) {
                                self.active_profile = Some(p.clone());
                            }
                        }
                    }
                    self.rename_profile_id = None;
                } else if cmd == "rename_cancel" {
                    self.rename_profile_id = None;
                }
            }
            
            ui.add_space(30.0);
            ui.label(egui::RichText::new("Create New Profile").size(24.0).strong());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Name:").size(18.0));
                ui.text_edit_singleline(&mut self.new_profile_name);
                if ui.button(egui::RichText::new("Create").size(18.0)).clicked() {
                    if !self.new_profile_name.trim().is_empty() {
                        if let Some(ref db) = self.db {
                            if let Ok(id) = db.create_profile(self.new_profile_name.trim(), "👶") {
                                self.profiles = db.get_profiles().unwrap_or_default();
                                if let Some(p) = self.profiles.iter().find(|p| p.id == id) {
                                    self.active_profile = Some(p.clone());
                                    self.screen = AppScreen::MainMenu;
                                }
                            }
                        }
                    }
                }
            });
            });
        });
    }

    fn show_statistics(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("◀ Back").size(20.0)).clicked() {
                self.screen = AppScreen::MainMenu;
            }
            ui.add_space(16.0);
            ui.label(egui::RichText::new("STATISTICS").size(32.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
        });
        ui.add_space(16.0);
        
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
            
            if let Some(ref p) = self.active_profile {
                if let Some(ref db) = self.db {
                    let played = db.get_stat(p.id, "games_played");
                    let won = db.get_stat(p.id, "games_won");
                    let win_rate = if played > 0 { (won as f64 / played as f64 * 100.0) as i64 } else { 0 };
                    
                    let best_square = db.get_stat(p.id, "best_time_square");
                    let best_hex = db.get_stat(p.id, "best_time_hex");
                    let total_playtime = db.get_stat(p.id, "total_playtime_s");
                    
                    let dark_text = egui::Color32::from_rgb(0x30, 0x30, 0x38);
                    
                    egui::Frame::new().fill(egui::Color32::from_rgb(0xFF, 0xFF, 0xFF)).inner_margin(24.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Games Played").size(16.0).color(dark_text));
                                ui.label(egui::RichText::new(played.to_string()).size(32.0).strong().color(dark_text));
                            });
                            ui.add_space(40.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Games Won").size(16.0).color(dark_text));
                                ui.label(egui::RichText::new(won.to_string()).size(32.0).strong().color(egui::Color32::from_rgb(0x40, 0xA0, 0x80)));
                            });
                            ui.add_space(40.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Win Rate").size(16.0).color(dark_text));
                                ui.label(egui::RichText::new(format!("{}%", win_rate)).size(32.0).strong().color(egui::Color32::from_rgb(0x79, 0x65, 0xBE)));
                            });
                        });
                    });
                    
                    ui.add_space(20.0);
                    
                    egui::Frame::new().fill(egui::Color32::from_rgb(0xFF, 0xFF, 0xFF)).inner_margin(24.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Total Playtime").size(16.0).color(dark_text));
                                let hours = total_playtime / 3600;
                                let mins = (total_playtime % 3600) / 60;
                                ui.label(egui::RichText::new(format!("{}h {}m", hours, mins)).size(24.0).strong().color(dark_text));
                            });
                            ui.add_space(40.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Best Square Time").size(16.0).color(dark_text));
                                let t_str = if best_square > 0 { format!("{}s", best_square) } else { "--".to_string() };
                                ui.label(egui::RichText::new(t_str).size(24.0).strong().color(dark_text));
                            });
                            ui.add_space(40.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Best Hex Time").size(16.0).color(dark_text));
                                let t_str = if best_hex > 0 { format!("{}s", best_hex) } else { "--".to_string() };
                                ui.label(egui::RichText::new(t_str).size(24.0).strong().color(dark_text));
                            });
                        });
                    });
                }
            }
            });
        });
    }

    fn show_achievements(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("◀ Back").size(20.0)).clicked() {
                self.screen = AppScreen::MainMenu;
            }
            ui.add_space(16.0);
            ui.label(egui::RichText::new("ACHIEVEMENTS").size(32.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
        });
        ui.add_space(16.0);
        
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(ref p) = self.active_profile {
                    if let Some(ref db) = self.db {
                        let unlocked = db.get_unlocked_achievements(p.id).unwrap_or_default();
                        
                        let achievements = vec![
                            ("first_win", "First Victory", "Win your first game."),
                            ("expert_clear", "Expert Solver", "Clear an Expert board."),
                            ("hex_master", "Hex Master", "Win a Hexagonal game."),
                            ("win_10", "Rising Star", "Win 10 games in total."),
                            ("win_50", "Dedicated Sweeper", "Win 50 games in total."),
                            ("win_100", "Centurion", "Win 100 games in total."),
                            ("win_500", "Mine Lord", "Win 500 games in total."),
                            ("play_10", "Initiate", "Play 10 games in total."),
                            ("play_100", "Enthusiast", "Play 100 games in total."),
                            ("play_1000", "Obsessed", "Play 1000 games in total."),
                            ("hex_10", "Hex Fanatic", "Win 10 Hexagonal games."),
                            ("hex_50", "Hex Whisperer", "Win 50 Hexagonal games."),
                            ("fast_60", "Speed Demon", "Win any game under 60 seconds."),
                            ("fast_30", "Lightning Fast", "Win any game under 30 seconds."),
                            ("fast_10", "Blink of an Eye", "Win any game under 10 seconds."),
                            ("playtime_1h", "Just One More", "Accumulate 1 hour of playtime."),
                            ("playtime_10h", "Time Well Spent", "Accumulate 10 hours of playtime."),
                            ("playtime_100h", "Chronos", "Accumulate 100 hours of playtime."),
                            ("chord_master", "Efficiency", "Win a game using chording extensively."),
                            ("no_flags", "Daredevil", "Win without placing any flags."),
                            ("lose_1", "First Blood", "Lose a game. It happens."),
                            ("lose_100", "Stubborn", "Lose 100 games. Keep trying!"),
                            ("first_hex", "New Dimensions", "Play your first Hexagonal game."),
                            ("level_5", "Level 5", "Reach profile level 5."),
                            ("level_10", "Level 10", "Reach profile level 10."),
                            ("level_50", "Level 50", "Reach profile level 50."),
                        ];
                        
                        for (id, title, desc) in achievements {
                            let is_unlocked = unlocked.contains(&id.to_string());
                            let bg_color = if is_unlocked { egui::Color32::from_rgb(0xFF, 0xD6, 0xA8) } else { egui::Color32::from_rgb(0xFF, 0xFF, 0xFF) };
                            let icon = if is_unlocked { "🏆" } else { "🔒" };
                            let text_color = if is_unlocked { egui::Color32::from_rgb(0x30, 0x30, 0x38) } else { egui::Color32::from_rgb(0xA1, 0xA0, 0xA8) };
                            
                            egui::Frame::new().fill(bg_color).inner_margin(16.0).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).size(32.0));
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(title).size(20.0).strong().color(text_color));
                                        ui.label(egui::RichText::new(desc).size(16.0).color(text_color));
                                    });
                                });
                            });
                            ui.add_space(10.0);
                        }
                    }
                }
                ui.add_space(40.0);
            });
        });
    }
}

impl eframe::App for MineShiftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if !matches!(self.screen, AppScreen::MainMenu) {
                if matches!(self.screen, AppScreen::Profiles) && self.active_profile.is_none() {
                    // Force selecting a profile
                } else {
                    self.screen = AppScreen::MainMenu;
                }
            }
        }
        
        let mut frame_style = egui::Frame::central_panel(ui.style());
        frame_style.fill = egui::Color32::from_rgb(0xEE, 0xE2, 0xFF); // Lavender background
        
        egui::CentralPanel::default().frame(frame_style).show(ui, |ui| {
            match self.screen {
                AppScreen::MainMenu => self.show_main_menu(ui),
                AppScreen::Gameplay => self.show_gameplay(ui),
                AppScreen::Profiles => self.show_profiles(ui),
                AppScreen::Statistics => self.show_statistics(ui),
                AppScreen::Achievements => self.show_achievements(ui),
            }
        });
    }
}
