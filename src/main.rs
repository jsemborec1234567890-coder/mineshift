mod db;
pub mod theme;
#[cfg(test)]
mod settings_tests;

use minecore::board::{Board, GameStatus, CellState};
use minecore::mode;
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
    GameModes,
    GeneratingNoGuess,
    ThemeStudio,
    ProfileCosmetics,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CustomCursorState {
    Default,
    UIHover,
    BoardHover,
    BoardFlag,
    Disabled,
    Loading,
    Victory,
    Defeat,
}

struct MineShiftApp {
    screen: AppScreen,
    cursor_state: CustomCursorState,
    board: Board,
    mode_state: mode::ModeState,
    db: Option<db::Database>,
    active_profile: Option<db::Profile>,
    profiles: Vec<db::Profile>,
    new_profile_name: String,
    rename_profile_id: Option<i64>,
    rename_buffer: String,
    generation_rx: Option<std::sync::mpsc::Receiver<(Option<Board>, usize, usize)>>,
    theme: crate::theme::Theme,
}

impl MineShiftApp {
    fn apply_profile_theme(&mut self) {
        if let Some(ref prof) = self.active_profile {
            if let Some(theme) = crate::theme::Theme::get_preset(&prof.selected_theme_id) {
                self.theme = theme;
            } else if let Some(ref db) = self.db {
                if let Ok(customs) = db.get_custom_themes(prof.id) {
                    if let Some((_, _, data)) = customs.into_iter().find(|(id, _, _)| id == &prof.selected_theme_id) {
                        if let Ok(mut t) = serde_json::from_str::<crate::theme::Theme>(&data) {
                            t.is_custom = true;
                            self.theme = t;
                        }
                    }
                }
            }
        }
    }
    fn new() -> Self {
        let db = db::Database::new().ok();
        let mut profiles = Vec::new();
        let mut active_profile = None;
        
        if let Some(ref d) = db {
            if let Ok(profs) = d.get_profiles() {
                if !profs.is_empty() {
                    active_profile = Some(profs[0].clone());
                }
                profiles = profs;
            }
        }
        
        let screen = if active_profile.is_some() { AppScreen::MainMenu } else { AppScreen::Profiles };

        let mut app = Self {
            screen,
            cursor_state: CustomCursorState::Default,
            board: Board::new_square(9, 9, 10),
            mode_state: mode::ModeState::new(mode::GameModeId::Classic),
            db,
            active_profile,
            profiles,
            new_profile_name: String::new(),
            rename_profile_id: None,
            rename_buffer: String::new(),
            generation_rx: None,
            theme: crate::theme::Theme::soft_circuit(),
        };
        app.apply_profile_theme();
        app
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
            
            let play_btn = egui::Button::new(
                egui::RichText::new("PLAY").size(20.0).color(egui::Color32::from_rgb(0xFF, 0xFF, 0xFF))
            ).fill(egui::Color32::from_rgb(121, 101, 190)).min_size(egui::vec2(400.0, 54.0));
            
            if ui.add(play_btn).clicked() {
                self.screen = AppScreen::GameModes;
            }
            
            ui.add_space(12.0);
            
            let customize_btn = egui::Button::new(
                egui::RichText::new("CUSTOMIZE").size(20.0).color(egui::Color32::from_rgb(0x30, 0x30, 0x38))
            ).fill(egui::Color32::from_rgb(208, 188, 255)).min_size(egui::vec2(400.0, 54.0));
            
            if ui.add(customize_btn).clicked() {
                self.screen = AppScreen::ThemeStudio;
            }
            
            ui.add_space(12.0);
            
            let secondary_fill = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
            let text_color = egui::Color32::from_rgb(0x4A, 0x4A, 0x4A);
            
            if ui.add(egui::Button::new(egui::RichText::new("PROFILES").size(18.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(400.0, 48.0))).clicked() {
                if let Some(ref d) = self.db {
                    self.profiles = d.get_profiles().unwrap_or_default();
                }
                self.screen = AppScreen::Profiles;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("STATISTICS").size(18.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(400.0, 48.0))).clicked() {
                self.screen = AppScreen::Statistics;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("ACHIEVEMENTS").size(18.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(400.0, 48.0))).clicked() {
                self.screen = AppScreen::Achievements;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("SETTINGS").size(18.0).color(text_color)).fill(secondary_fill).min_size(egui::vec2(400.0, 48.0))).clicked() {
                self.screen = AppScreen::Settings;
            }
            ui.add_space(12.0);
            if ui.add(egui::Button::new(egui::RichText::new("EXIT").size(18.0).color(egui::Color32::from_rgb(0xE8, 0x5C, 0x60))).fill(secondary_fill).min_size(egui::vec2(400.0, 48.0))).clicked() {
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
                    minecore::board::Geometry::Square => format!("SQUARE {}×{}", self.board.width, self.board.height),
                    minecore::board::Geometry::Hex => format!("HEXAGON R{}", self.board.radius),
                };
                ui.label(egui::RichText::new(mode_text).size(16.0).color(egui::Color32::from_rgb(0xA1, 0xA0, 0xA8)));
                
                ui.add_space(20.0);
                
                let elapsed = if let Some(end) = self.board.end_time {
                    if let Some(start) = self.board.start_time {
                        end.duration_since(start).as_secs_f64()
                    } else { 0.0 }
                } else if let Some(start) = self.board.start_time {
                    std::time::Instant::now().duration_since(start).as_secs_f64()
                } else { 0.0 };

                let time_display = if let Some(limit) = self.mode_state.time_limit_sec {
                    let remaining = (limit - elapsed).max(0.0);
                    if remaining <= 0.0 && self.board.status == GameStatus::Playing {
                        self.board.status = GameStatus::Lost;
                        self.board.end_time = Some(std::time::Instant::now());
                    }
                    format!("⏱ {:03.0}", remaining)
                } else {
                    format!("⏱ {:03.0}", elapsed)
                };

                ui.label(egui::RichText::new(time_display).size(24.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
                
                ui.add_space(20.0);
                
                if matches!(self.mode_state.id, mode::GameModeId::Blitz | mode::GameModeId::Sandbox) {
                    ui.label(egui::RichText::new(format!("⭐ {}", self.mode_state.score)).size(24.0).strong().color(egui::Color32::from_rgb(0x79, 0x65, 0xBE)));
                    ui.add_space(20.0);
                }
                
                if matches!(self.mode_state.id, mode::GameModeId::Survival | mode::GameModeId::Expedition) {
                    ui.label(egui::RichText::new(format!("❤️ {}", self.mode_state.lives)).size(24.0).strong().color(egui::Color32::from_rgb(0xE8, 0x5C, 0x60)));
                    ui.add_space(20.0);
                }
                
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
                            minecore::board::Geometry::Square => Board::new_square(self.board.width, self.board.height, self.board.mine_count),
                            minecore::board::Geometry::Hex => Board::new_hex(self.board.radius, self.board.mine_count),
                        };
                        self.board.is_no_guess = self.mode_state.id == mode::GameModeId::NoGuess;
                    }
                    
                    if self.mode_state.id == mode::GameModeId::Zen && self.board.status == GameStatus::Playing {
                        if ui.button(egui::RichText::new("Hint").size(18.0)).clicked() {
                            let deductions = minecore::solver::Solver::deduce(&self.board);
                            for d in deductions {
                                match d {
                                    minecore::solver::Deduction::Safe(x, y) => {
                                        let idx = self.board.get_index(x, y);
                                        self.board.cells[idx].state = CellState::Revealed;
                                    }
                                    minecore::solver::Deduction::Mine(x, y) => {
                                        let idx = self.board.get_index(x, y);
                                        self.board.cells[idx].state = CellState::Flagged;
                                    }
                                }
                            }
                        }
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
                    minecore::board::Geometry::Square => (base_cell_size + base_spacing) * self.board.width as f32,
                    minecore::board::Geometry::Hex => (base_cell_size * 3.0_f32.sqrt() + base_spacing) * self.board.width as f32,
                };
                let raw_h = match self.board.geometry {
                    minecore::board::Geometry::Square => (base_cell_size + base_spacing) * self.board.height as f32,
                    minecore::board::Geometry::Hex => (base_cell_size * 1.5 + base_spacing) * self.board.height as f32,
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
                                minecore::board::Geometry::Square => {
                                    rect.min + egui::vec2(
                                        x as f32 * (cell_size + spacing) + cell_size / 2.0,
                                        y as f32 * (cell_size + spacing) + cell_size / 2.0
                                    )
                                }
                                minecore::board::Geometry::Hex => {
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
                                    if cell.state == minecore::board::CellState::Flagged {
                                        self.cursor_state = CustomCursorState::BoardFlag;
                                    } else if cell.state == minecore::board::CellState::Covered {
                                        self.cursor_state = CustomCursorState::BoardHover;
                                    } else {
                                        self.cursor_state = CustomCursorState::Default;
                                    }
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
                                minecore::board::Geometry::Square => {
                                    let cell_rect = egui::Rect::from_center_size(center, egui::vec2(cell_size, cell_size));
                                    let shadow_rect = cell_rect.translate(egui::vec2(0.0, 2.0));
                                    ui.painter().rect_filled(shadow_rect, 6.0, egui::Color32::from_black_alpha(20));
                                    ui.painter().rect_filled(cell_rect, 6.0, fill_color);
                                }
                                minecore::board::Geometry::Hex => {
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
                    if let Some((x, y)) = clicked_cell { 
                        if self.board.first_click && self.board.is_no_guess {
                            // Start generation thread
                            let board_clone = self.board.clone();
                            let (tx, rx) = std::sync::mpsc::channel();
                            self.generation_rx = Some(rx);
                            self.screen = AppScreen::GeneratingNoGuess;
                            
                            std::thread::spawn(move || {
                                let result = minecore::no_guess::generate_no_guess_board(board_clone, x, y);
                                let _ = tx.send((result, x, y));
                            });
                        } else {
                            self.board.reveal(x, y); 
                        }
                    }
                    if let Some((x, y)) = secondary_clicked_cell { self.board.toggle_flag(x, y); }
                    if let Some((x, y)) = middle_clicked_cell { self.board.chord(x, y); }
                    
                    if old_status == GameStatus::Playing && self.board.status != GameStatus::Playing {
                        let mut true_end = true;

                        match self.mode_state.id {
                            mode::GameModeId::Blitz => {
                                if self.board.status == GameStatus::Won {
                                    self.mode_state.score += 500;
                                } else {
                                    self.mode_state.score = (self.mode_state.score - 150).max(0);
                                }
                                if self.mode_state.time_limit_sec.unwrap_or(0.0) - self.board.start_time.unwrap().elapsed().as_secs_f64() > 0.0 {
                                    // Instant restart for Blitz
                                    self.board = match self.board.geometry {
                                        minecore::board::Geometry::Square => Board::new_square(self.board.width, self.board.height, self.board.mine_count),
                                        minecore::board::Geometry::Hex => Board::new_hex(self.board.radius, self.board.mine_count),
                                    };
                                    true_end = false;
                                }
                            }
                            mode::GameModeId::Survival => {
                                if self.board.status == GameStatus::Won {
                                    self.mode_state.stage += 1;
                                    self.board = match self.board.geometry {
                                        minecore::board::Geometry::Square => Board::new_square(self.board.width, self.board.height, self.board.mine_count + 2),
                                        minecore::board::Geometry::Hex => Board::new_hex(self.board.radius, self.board.mine_count + 2),
                                    };
                                    true_end = false;
                                } else {
                                    self.mode_state.lives -= 1;
                                    if self.mode_state.lives > 0 {
                                        self.board = match self.board.geometry {
                                            minecore::board::Geometry::Square => Board::new_square(self.board.width, self.board.height, self.board.mine_count),
                                            minecore::board::Geometry::Hex => Board::new_hex(self.board.radius, self.board.mine_count),
                                        };
                                        true_end = false;
                                    }
                                }
                            }
                            mode::GameModeId::Expedition => {
                                if self.board.status == GameStatus::Won {
                                    if self.mode_state.stage < 30 {
                                        self.mode_state.stage += 1;
                                        self.board = match self.board.geometry {
                                            minecore::board::Geometry::Square => Board::new_square(self.board.width, self.board.height, (20 + self.mode_state.stage).min(60) as usize),
                                            minecore::board::Geometry::Hex => Board::new_hex(self.board.radius, (20 + self.mode_state.stage).min(60) as usize),
                                        };
                                        self.board.seed = Some(self.mode_state.stage as u64);
                                        if let (Some(db), Some(profile)) = (&self.db, &self.active_profile) {
                                            // Ensure we don't regress if they replay a lower stage
                                            let current = db.get_stat(profile.id, "expedition_stage");
                                            if self.mode_state.stage as i64 > current {
                                                let _ = db.set_highest_stat(profile.id, "expedition_stage", self.mode_state.stage as i64);
                                            }
                                        }
                                        true_end = false;
                                    }
                                }
                            }
                            _ => {}
                        }

                        if true_end {
                            if let (Some(db), Some(profile)) = (&self.db, &self.active_profile) {
                                let duration_ms = if let (Some(s), Some(e)) = (self.board.start_time, self.board.end_time) {
                                    e.duration_since(s).as_millis() as i64
                                } else { 0 };
                                let started_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 - (duration_ms / 1000);
                                
                                let session = db::GameSession {
                                    profile_id: profile.id,
                                    mode: self.mode_state.id.name().to_string(),
                                    geometry: match self.board.geometry { minecore::board::Geometry::Square => "Square".to_string(), minecore::board::Geometry::Hex => "Hex".to_string() },
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
                                    if self.board.geometry == minecore::board::Geometry::Square {
                                        let _ = db.set_best_time(profile.id, "best_time_square", duration_ms / 1000);
                                        let _ = db.unlock_achievement(profile.id, "first_win");
                                        if self.board.mine_count >= 99 {
                                            let _ = db.unlock_achievement(profile.id, "expert_clear");
                                        }
                                    } else {
                                        let _ = db.set_best_time(profile.id, "best_time_hex", duration_ms / 1000);
                                        let _ = db.unlock_achievement(profile.id, "hex_master");
                                    }
                                    
                                    if self.mode_state.id == mode::GameModeId::Blitz && self.mode_state.score >= 2000 {
                                        let _ = db.unlock_achievement(profile.id, "blitz_star");
                                    }
                                    if self.mode_state.id == mode::GameModeId::Survival && self.mode_state.stage >= 5 {
                                        let _ = db.unlock_achievement(profile.id, "survival_5");
                                    }
                                    if self.mode_state.id == mode::GameModeId::Expedition && self.mode_state.stage >= 10 {
                                        let _ = db.unlock_achievement(profile.id, "expedition_10");
                                    }
                                    if self.mode_state.id == mode::GameModeId::Daily {
                                        let _ = db.unlock_achievement(profile.id, "daily_win");
                                    }
                                }
                                self.profiles = db.get_profiles().unwrap_or_default();
                                if let Some(p) = self.profiles.iter().find(|p| p.id == profile.id) {
                                    self.active_profile = Some(p.clone());
                                }
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
                if ui.button(egui::RichText::new("✨ Customize Cosmetics").size(20.0)).clicked() {
                    self.screen = AppScreen::ProfileCosmetics;
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
                            ("blitz_star", "Blitz Star", "Score 2000 points in Blitz mode."),
                            ("survival_5", "Survivor", "Reach Stage 5 in Survival mode."),
                            ("expedition_10", "Explorer", "Reach Stage 10 in Expedition mode."),
                            ("daily_win", "Daily Challenger", "Win a Daily Challenge."),
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

    fn show_game_modes(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("◀ Back").size(20.0)).clicked() {
                self.screen = AppScreen::MainMenu;
            }
            ui.add_space(16.0);
            ui.label(egui::RichText::new("GAME MODES").size(32.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
        });
        ui.add_space(16.0);
        
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let modes = vec![
                    (mode::GameModeId::Classic, "⬜", "SQUARE"),
                    (mode::GameModeId::Classic, "⬡", "HEXAGON"),
                    (mode::GameModeId::Sprint, "⚡", "SQUARE"),
                    (mode::GameModeId::NoGuess, "🧠", "SQUARE"),
                    (mode::GameModeId::Blitz, "🔥", "SQUARE"),
                    (mode::GameModeId::Survival, "❤️", "SQUARE"),
                    (mode::GameModeId::Daily, "📅", "SQUARE"),
                    (mode::GameModeId::Expedition, "🗺️", "SQUARE"),
                    (mode::GameModeId::Zen, "🧘", "SQUARE"),
                    (mode::GameModeId::Sandbox, "⚙️", "SQUARE"),
                ];
                
                for (id, icon, variant) in modes {
                    egui::Frame::new().fill(egui::Color32::from_rgb(0xFF, 0xFF, 0xFF)).inner_margin(16.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).size(32.0));
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(format!("{} - {}", id.name(), variant)).size(24.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
                                ui.label(egui::RichText::new(id.desc()).size(16.0).color(egui::Color32::from_rgb(0xA1, 0xA0, 0xA8)));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new(egui::RichText::new("PLAY").size(20.0)).fill(egui::Color32::from_rgb(0x79, 0x65, 0xBE))).clicked() {
                                    self.mode_state = mode::ModeState::new(id.clone());
                                    if variant == "SQUARE" {
                                        self.board = Board::new_square(16, 16, 40); // Intermediate
                                    } else {
                                        self.board = Board::new_hex(8, 40);
                                    }
                                    
                                    if self.mode_state.id == mode::GameModeId::NoGuess {
                                        self.board.is_no_guess = true;
                                    } else if self.mode_state.id == mode::GameModeId::Blitz {
                                        self.mode_state.time_limit_sec = Some(180.0);
                                        self.mode_state.score = 0;
                                    } else if self.mode_state.id == mode::GameModeId::Survival {
                                        self.mode_state.lives = 3;
                                        self.mode_state.stage = 1;
                                    } else if self.mode_state.id == mode::GameModeId::Expedition {
                                        self.mode_state.lives = 1;
                                        if let (Some(db), Some(profile)) = (&self.db, &self.active_profile) {
                                            self.mode_state.stage = db.get_stat(profile.id, "expedition_stage").max(1) as i32;
                                        } else {
                                            self.mode_state.stage = 1;
                                        }
                                        self.board.seed = Some(self.mode_state.stage as u64);
                                        self.board.mine_count = (20 + self.mode_state.stage).min(60) as usize;
                                    } else if self.mode_state.id == mode::GameModeId::Daily {
                                        use std::hash::{Hash, Hasher};
                                        use std::collections::hash_map::DefaultHasher;
                                        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                                        let days_since_epoch = now / 86400;
                                        let mut hasher = DefaultHasher::new();
                                        days_since_epoch.hash(&mut hasher);
                                        self.board.seed = Some(hasher.finish());
                                    }

                                    self.screen = AppScreen::Gameplay;
                                }
                            });
                        });
                    });
                    ui.add_space(16.0);
                }
            });
        });
    }
    fn show_generating(&mut self, ui: &mut egui::Ui) {
        // Must request repaint continuously while generating to keep the UI ticking
        ui.ctx().request_repaint();

        if let Some(rx) = &self.generation_rx {
            if let Ok((result, cx, cy)) = rx.try_recv() {
                if let Some(mut new_b) = result {
                    new_b.start_time = Some(std::time::Instant::now());
                    new_b.is_no_guess = true;
                    // First click was consumed by generator
                    new_b.first_click = false;
                    self.board = new_b;
                } else {
                    // Fallback to normal mine placement.
                    self.board.is_no_guess = false;
                    self.board.reveal(cx, cy);
                }
                self.screen = AppScreen::Gameplay;
                self.generation_rx = None;
                return;
            }
        }

        ui.vertical_centered(|ui| {
            ui.add_space(200.0);
            ui.label(egui::RichText::new("Preparing a logically solvable puzzle...").size(24.0).strong().color(egui::Color32::from_rgb(0x30, 0x30, 0x38)));
            ui.add_space(32.0);
            ui.spinner();
            ui.add_space(64.0);
            if ui.button(egui::RichText::new("Cancel").size(20.0)).clicked() {
                self.generation_rx = None;
                self.screen = AppScreen::MainMenu;
            }
        });
    }
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
            ui.label(egui::RichText::new("CUSTOMIZATION HUB").size(24.0).strong().color(self.theme.colors.text_primary()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("✨ Profile Cosmetics").size(16.0)).clicked() {
                    self.screen = AppScreen::ProfileCosmetics;
                }
            });
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
                    ui.add(egui::Slider::new(&mut self.theme.geometry.animation_speed, 0.0..=5.0).text("Animation Speed"));
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
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = AppScreen::MainMenu;
            }
            ui.label(egui::RichText::new("SETTINGS").size(24.0).strong().color(self.theme.colors.text_primary()));
        });
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Cursor & Accessibility");
            ui.checkbox(&mut self.theme.cursor.enabled, "Enable Custom Cursor");
            if self.theme.cursor.enabled {
                ui.add(egui::Slider::new(&mut self.theme.cursor.size, 0.5..=3.0).text("Cursor Size"));
                ui.checkbox(&mut self.theme.cursor.trail, "Enable Trail");
            }
            ui.checkbox(&mut self.theme.cursor.reduced_motion, "Reduced Motion (Disables Cursor Trail)");
            
            ui.add_space(16.0);
            ui.heading("Animations");
            ui.add(egui::Slider::new(&mut self.theme.geometry.animation_speed, 0.0..=5.0).text("Animation Speed"));
            
            ui.add_space(32.0);
            if ui.button("Save Settings to Profile").clicked() {
                if let (Some(prof), Some(db)) = (&self.active_profile, &self.db) {
                    if self.theme.is_custom {
                        let data = serde_json::to_string(&self.theme).unwrap();
                        let _ = db.save_custom_theme(prof.id, &self.theme.id, &self.theme.name, &data);
                    } else {
                        let _ = db.set_profile_theme(prof.id, &self.theme.id);
                    }
                }
            }
        });
    }

    fn show_profile_cosmetics(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = AppScreen::ThemeStudio;
            }
            ui.label(egui::RichText::new("PROFILE COSMETICS").size(24.0).strong().color(self.theme.colors.text_primary()));
        });
        ui.separator();
        
        let mut updated = false;
        if let Some(ref mut prof) = self.active_profile {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(200.0);
                    ui.heading("Avatars");
                    let avatars = vec!["🧑", "👽", "🤖", "🎭", "💾", "⚡", "🔮", "🎮"];
                    for avatar in avatars {
                        if ui.selectable_label(prof.avatar_id == avatar, avatar).clicked() {
                            prof.avatar_id = avatar.to_string();
                            updated = true;
                        }
                    }
                    
                    ui.separator();
                    ui.heading("Badges");
                    let badges = vec!["None", "🌟 Pioneer", "🔥 Streak", "🏆 Champion", "⚔️ Veteran"];
                    for badge in badges {
                        if ui.selectable_label(prof.badge_id == badge, badge).clicked() {
                            prof.badge_id = badge.to_string();
                            updated = true;
                        }
                    }
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.heading("Titles");
                    let titles = vec!["Novice Sweeper", "Logic Master", "Hexadecimal", "Mine Specialist", "The Untouchable", "Circuit Breaker"];
                    for title in titles {
                        if ui.selectable_label(prof.title == title, title).clicked() {
                            prof.title = title.to_string();
                            updated = true;
                        }
                    }
                    
                    ui.separator();
                    ui.heading("Preview");
                    egui::Frame::new().fill(self.theme.colors.card_bg()).inner_margin(16.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&prof.avatar_id).size(48.0));
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&prof.display_name).size(24.0).strong().color(self.theme.colors.text_primary()));
                                ui.label(egui::RichText::new(&prof.title).size(16.0).color(self.theme.colors.accent()));
                                if prof.badge_id != "None" && !prof.badge_id.is_empty() {
                                    ui.label(egui::RichText::new(&prof.badge_id).size(16.0).color(self.theme.colors.success()));
                                }
                            });
                        });
                    });
                });
            });
            
            if updated {
                if let Some(ref db) = self.db {
                    let _ = db.set_profile_cosmetics(prof.id, &prof.title, &prof.badge_id);
                    let _ = db.set_profile_avatar(prof.id, &prof.avatar_id);
                }
            }
        } else {
            ui.label("No active profile.");
        }
    }
}

fn color_picker(ui: &mut eframe::egui::Ui, color: &mut (u8, u8, u8)) {
    let mut c = [color.0, color.1, color.2];
    ui.color_edit_button_srgb(&mut c);
    *color = (c[0], c[1], c[2]);
}

impl eframe::App for MineShiftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.cursor_state = CustomCursorState::Default;
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
                AppScreen::GameModes => self.show_game_modes(ui),
                AppScreen::GeneratingNoGuess => self.show_generating(ui),
                AppScreen::ThemeStudio => self.show_theme_studio(ui),
                AppScreen::ProfileCosmetics => self.show_profile_cosmetics(ui),
                AppScreen::Settings => self.show_settings(ui),
            }
        });
    }
}
