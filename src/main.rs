mod audio;
mod coords;
mod db;
mod juice;
mod particles;
pub mod settings;
#[cfg(test)]
mod settings_tests;
pub mod theme;
mod ui_tokens;

use audio::SoundEffect;

use eframe::egui;
use minecore::board::{Board, CellState, GameStatus};
use minecore::challenge::{ChallengeTier, MasteryChallenge};
use minecore::mode;
use minecore::rules::{
    AssistPolicy, CluePolicy, FirstClickPolicy, FlagPolicy, LogicGuarantee, Ruleset,
};
use minecore::solver::{DeductionType, Explanation, Solver};

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    {
        if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
            fonts.font_data.insert(
                "segoe_ui".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                prop.insert(0, "segoe_ui".to_owned());
            }
        }
        if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
            fonts.font_data.insert(
                "segoe_ui_symbol".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                prop.push("segoe_ui_symbol".to_owned());
            }
        }
    }

    ctx.set_fonts(fonts);
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut width = 1000.0;
    let mut height = 700.0;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--width" && i + 1 < args.len() {
            if let Ok(w) = args[i + 1].parse::<f32>() {
                width = w;
            }
        }
        if args[i] == "--height" && i + 1 < args.len() {
            if let Ok(h) = args[i + 1].parse::<f32>() {
                height = h;
            }
        }
        i += 1;
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("MINE//SHIFT: BEYOND")
        .with_inner_size([width, height])
        .with_min_inner_size([600.0, 400.0]);

    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "MINE//SHIFT",
        options,
        Box::new(move |cc| {
            configure_fonts(&cc.egui_ctx);
            let mut app = MineShiftApp::new();
            app.apply_cli_args(&args);
            Ok(Box::new(app))
        }),
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
    MasteryChallenges,
    SandboxLab,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    General,
    Audio,
    GameFeel,
    Cursor,
    Appearance,
    Accessibility,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeStudioTab {
    Colors,
    Geometry,
    Numbers,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AchievementFilter {
    All,
    Unlocked,
    Locked,
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
    settings: crate::settings::UserSettings,
    settings_tab: SettingsTab,
    settings_saved_feedback: Option<f64>,
    theme_studio_tab: ThemeStudioTab,
    achievement_filter: AchievementFilter,
    audio: crate::audio::SoundManager,
    particles: crate::particles::ParticleSystem,
    shake: crate::juice::TraumaShake,
    anim_tracker: crate::juice::CellAnimationTracker,
    last_frame_time: f64,
    screenshot_target: Option<String>,
    frame_counter: u32,
    challenge_tier_filter: Option<ChallengeTier>,
    active_challenge_id: Option<String>,
    challenge_completions: std::collections::HashMap<String, db::ChallengeCompletion>,
    active_hint: Option<Explanation>,
    sandbox_geometry: minecore::board::Geometry,
    sandbox_width: usize,
    sandbox_height: usize,
    sandbox_radius: usize,
    sandbox_mines: usize,
    sandbox_opening_policy: FirstClickPolicy,
    sandbox_flag_policy: FlagPolicy,
    sandbox_clue_policy: CluePolicy,
    sandbox_logic_guarantee: LogicGuarantee,
    blitz_combo: u32,
    blitz_last_reveal_time: f64,
}

impl MineShiftApp {
    fn sync_settings(&mut self) {
        self.audio.master_volume = self.settings.master_volume;
        self.audio.sfx_volume = self.settings.sfx_volume;
        self.audio.muted = self.settings.mute_audio;
        self.audio.pitch_variation = self.settings.pitch_variation;
        self.particles.enabled = self.settings.particles_enabled;
        self.shake.intensity = if self.settings.cursor.reduced_motion {
            0.0
        } else {
            self.settings.screen_shake
        };
        self.theme.cursor = self.settings.cursor.clone();
        self.theme.geometry.animation_speed = self.settings.animation_speed;
    }

    fn load_profile_settings(&mut self) {
        if let (Some(prof), Some(db)) = (&self.active_profile, &self.db) {
            if let Ok(Some(json)) = db.get_profile_settings(prof.id) {
                if let Ok(s) = serde_json::from_str::<crate::settings::UserSettings>(&json) {
                    self.settings = s;
                }
            }
        }
        self.sync_settings();
    }

    fn save_current_settings(&self) {
        if let (Some(prof), Some(db)) = (&self.active_profile, &self.db) {
            if let Ok(json) = serde_json::to_string(&self.settings) {
                let _ = db.save_profile_settings(prof.id, &json);
            }
            let _ = db.set_profile_theme(prof.id, &self.theme.id);
        }
    }

    fn set_active_theme(&mut self, theme_id: &str) {
        if let Some(mut t) = crate::theme::Theme::get_preset(theme_id) {
            t.cursor = self.settings.cursor.clone();
            t.geometry.animation_speed = self.settings.animation_speed;
            self.theme = t;
            if let Some(ref mut prof) = self.active_profile {
                prof.selected_theme_id = theme_id.to_string();
                if let Some(ref db) = self.db {
                    let _ = db.set_profile_theme(prof.id, theme_id);
                }
            }
            if let Some(ref active) = self.active_profile {
                if let Some(p) = self.profiles.iter_mut().find(|p| p.id == active.id) {
                    p.selected_theme_id = theme_id.to_string();
                }
            }
        }
    }

    fn apply_profile_theme(&mut self) {
        if let Some(ref prof) = self.active_profile {
            if let Some(theme) = crate::theme::Theme::get_preset(&prof.selected_theme_id) {
                self.theme = theme;
            } else if let Some(ref db) = self.db {
                if let Ok(customs) = db.get_custom_themes(prof.id) {
                    if let Some((_, _, data)) = customs
                        .into_iter()
                        .find(|(id, _, _)| id == &prof.selected_theme_id)
                    {
                        if let Ok(mut t) = serde_json::from_str::<crate::theme::Theme>(&data) {
                            t.is_custom = true;
                            self.theme = t;
                        }
                    }
                }
            }
        }
        self.theme.cursor = self.settings.cursor.clone();
        self.theme.geometry.animation_speed = self.settings.animation_speed;
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

        let screen = if active_profile.is_some() {
            AppScreen::MainMenu
        } else {
            AppScreen::Profiles
        };

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
            settings: crate::settings::UserSettings::default(),
            settings_tab: SettingsTab::Audio,
            settings_saved_feedback: None,
            theme_studio_tab: ThemeStudioTab::Colors,
            achievement_filter: AchievementFilter::All,
            audio: crate::audio::SoundManager::new(),
            particles: crate::particles::ParticleSystem::new(),
            shake: crate::juice::TraumaShake::new(),
            anim_tracker: crate::juice::CellAnimationTracker::new(),
            last_frame_time: 0.0,
            screenshot_target: None,
            frame_counter: 0,
            challenge_tier_filter: None,
            active_challenge_id: None,
            challenge_completions: std::collections::HashMap::new(),
            active_hint: None,
            sandbox_geometry: minecore::board::Geometry::Square,
            sandbox_width: 16,
            sandbox_height: 16,
            sandbox_radius: 6,
            sandbox_mines: 40,
            sandbox_opening_policy: FirstClickPolicy::GuaranteedZeroOpening,
            sandbox_flag_policy: FlagPolicy::Standard,
            sandbox_clue_policy: CluePolicy::StandardOnly,
            sandbox_logic_guarantee: LogicGuarantee::StandardProbabilistic,
            blitz_combo: 0,
            blitz_last_reveal_time: 0.0,
        };
        app.load_profile_settings();
        app.apply_profile_theme();
        if let (Some(prof), Some(db)) = (&app.active_profile, &app.db) {
            if let Ok(comps) = db.get_challenge_completions(prof.id) {
                app.challenge_completions = comps;
            }
        }
        app
    }

    pub fn apply_cli_args(&mut self, args: &[String]) {
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--screen" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "settings-cursor" => {
                                self.screen = AppScreen::Settings;
                                self.settings_tab = SettingsTab::Cursor;
                            }
                            "settings-gamefeel" => {
                                self.screen = AppScreen::Settings;
                                self.settings_tab = SettingsTab::GameFeel;
                            }
                            "settings-audio" => {
                                self.screen = AppScreen::Settings;
                                self.settings_tab = SettingsTab::Audio;
                            }
                            "settings-general" => {
                                self.screen = AppScreen::Settings;
                                self.settings_tab = SettingsTab::General;
                            }
                            "settings-appearance" => {
                                self.screen = AppScreen::Settings;
                                self.settings_tab = SettingsTab::Appearance;
                            }
                            "settings-accessibility" => {
                                self.screen = AppScreen::Settings;
                                self.settings_tab = SettingsTab::Accessibility;
                            }
                            "statistics" => {
                                self.screen = AppScreen::Statistics;
                            }
                            "achievements" => {
                                self.screen = AppScreen::Achievements;
                            }
                            "mainmenu" | "main-menu" => {
                                self.screen = AppScreen::MainMenu;
                            }
                            "gamemodes" | "game-modes" => {
                                self.screen = AppScreen::GameModes;
                            }
                            "profiles" => {
                                self.screen = AppScreen::Profiles;
                            }
                            "profile-cosmetics" | "cosmetics" => {
                                self.screen = AppScreen::ProfileCosmetics;
                            }
                            "themestudio" | "theme-studio" | "customize" => {
                                self.screen = AppScreen::ThemeStudio;
                            }
                            "mastery" | "mastery-challenges" | "challenges" => {
                                self.screen = AppScreen::MasteryChallenges;
                            }
                            "sandbox" | "sandbox-lab" => {
                                self.screen = AppScreen::SandboxLab;
                            }
                            "gameplay" | "gameplay-square" => {
                                self.screen = AppScreen::Gameplay;
                                self.board = Board::new_square(9, 9, 10);
                            }
                            "gameplay-hex" => {
                                self.screen = AppScreen::Gameplay;
                                self.board = Board::new_hex(5, 10);
                            }
                            _ => {}
                        }
                    }
                }
                "--theme-tab" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "colors" => self.theme_studio_tab = ThemeStudioTab::Colors,
                            "geometry" => self.theme_studio_tab = ThemeStudioTab::Geometry,
                            "numbers" => self.theme_studio_tab = ThemeStudioTab::Numbers,
                            _ => {}
                        }
                    }
                }
                "--achieve-filter" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "all" => self.achievement_filter = AchievementFilter::All,
                            "unlocked" => self.achievement_filter = AchievementFilter::Unlocked,
                            "locked" => self.achievement_filter = AchievementFilter::Locked,
                            _ => {}
                        }
                    }
                }
                "--mode" => {
                    if i + 1 < args.len() {
                        i += 1;
                        let mode_id = match args[i].as_str() {
                            "classic" => mode::GameModeId::Classic,
                            "sprint" => mode::GameModeId::Sprint,
                            "noguess" => mode::GameModeId::NoGuess,
                            "blitz" => mode::GameModeId::Blitz,
                            "survival" => mode::GameModeId::Survival,
                            "daily" => mode::GameModeId::Daily,
                            "expedition" => mode::GameModeId::Expedition,
                            "zen" => mode::GameModeId::Zen,
                            "sandbox" => mode::GameModeId::Sandbox,
                            _ => mode::GameModeId::Classic,
                        };
                        self.mode_state = mode::ModeState::new(mode_id);
                        self.screen = AppScreen::Gameplay;
                    }
                }
                "--geometry" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "square" => self.board = Board::new_square(9, 9, 10),
                            "hex" => self.board = Board::new_hex(5, 10),
                            _ => {}
                        }
                    }
                }
                "--board-size" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "beginner" => self.board = Board::new_square(9, 9, 10),
                            "intermediate" => self.board = Board::new_square(16, 16, 40),
                            "expert" => self.board = Board::new_square(30, 16, 99),
                            "hex-small" => self.board = Board::new_hex(3, 10),
                            "hex-med" => self.board = Board::new_hex(5, 20),
                            "hex-large" => self.board = Board::new_hex(7, 45),
                            _ => {}
                        }
                    }
                }
                "--game-state" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "partial" => {
                                self.board.first_click = false;
                                self.board.start_time = Some(std::time::Instant::now());
                                if self.board.cells.len() > 10 {
                                    self.board.cells[0].state = CellState::Revealed;
                                    self.board.cells[0].adjacent_mines = 0;
                                    self.board.cells[1].state = CellState::Revealed;
                                    self.board.cells[1].adjacent_mines = 1;
                                    self.board.cells[2].state = CellState::Revealed;
                                    self.board.cells[2].adjacent_mines = 2;
                                    self.board.cells[3].state = CellState::Flagged;
                                    self.board.flags_placed = 1;
                                }
                            }
                            "won" => {
                                self.board.status = GameStatus::Won;
                                self.board.start_time = Some(std::time::Instant::now());
                                self.board.end_time = Some(std::time::Instant::now());
                                for cell in &mut self.board.cells {
                                    if !cell.is_mine {
                                        cell.state = CellState::Revealed;
                                    } else {
                                        cell.state = CellState::Flagged;
                                    }
                                }
                                self.board.flags_placed = self.board.mine_count;
                            }
                            "lost" => {
                                self.board.status = GameStatus::Lost;
                                self.board.start_time = Some(std::time::Instant::now());
                                self.board.end_time = Some(std::time::Instant::now());
                                for cell in &mut self.board.cells {
                                    if cell.is_mine {
                                        cell.state = CellState::Revealed;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "--theme" => {
                    if i + 1 < args.len() {
                        i += 1;
                        self.set_active_theme(&args[i]);
                    }
                }
                "--cursor-size" => {
                    if i + 1 < args.len() {
                        i += 1;
                        if let Ok(sz) = args[i].parse::<f32>() {
                            self.settings.cursor.size = sz;
                            self.theme.cursor.size = sz;
                        }
                    }
                }
                "--saved-feedback" => {
                    self.settings_saved_feedback = Some(1e9);
                }
                "--trigger-shake" => {
                    self.settings.screen_shake = 1.0;
                    self.shake.intensity = 1.0;
                    self.shake.add_trauma(0.85);
                }
                "--screenshot" => {
                    if i + 1 < args.len() {
                        i += 1;
                        self.screenshot_target = Some(args[i].clone());
                    }
                }
                "--challenge" => {
                    if i + 1 < args.len() {
                        i += 1;
                        let chal_id = args[i].clone();
                        if let Some(ch) = MasteryChallenge::catalog()
                            .into_iter()
                            .find(|c| c.id == chal_id)
                        {
                            self.active_challenge_id = Some(ch.id.clone());
                            self.board = Board::from_ruleset(ch.ruleset.clone());
                            self.mode_state = mode::ModeState::new(mode::GameModeId::Classic);
                            if let Some(t) = ch.target_time_sec {
                                self.mode_state.time_limit_sec = Some(t);
                            }
                            self.screen = AppScreen::Gameplay;
                        }
                    }
                }
                "--clue-policy" => {
                    if i + 1 < args.len() {
                        i += 1;
                        if args[i] == "ring2" || args[i] == "extended" {
                            self.board.ruleset.clue_policy = CluePolicy::ExtendedRing2;
                        }
                    }
                }
                "--flag-policy" => {
                    if i + 1 < args.len() {
                        i += 1;
                        if args[i] == "no-flags" || args[i] == "noflags" {
                            self.board.ruleset.flag_policy = FlagPolicy::NoFlags;
                        } else if args[i].starts_with("limited:") {
                            if let Ok(limit) = args[i]["limited:".len()..].parse::<usize>() {
                                self.board.ruleset.flag_policy =
                                    FlagPolicy::LimitedFlags { max_flags: limit };
                            }
                        }
                    }
                }
                "--opening-policy" => {
                    if i + 1 < args.len() {
                        i += 1;
                        match args[i].as_str() {
                            "zero" => {
                                self.board.ruleset.first_click_policy =
                                    FirstClickPolicy::GuaranteedZeroOpening
                            }
                            "deduction" => {
                                self.board.ruleset.first_click_policy =
                                    FirstClickPolicy::ValidatedDeductionOpening
                            }
                            "standard" => {
                                self.board.ruleset.first_click_policy =
                                    FirstClickPolicy::StandardSafe
                            }
                            _ => {}
                        }
                    }
                }
                "--hint" => {
                    self.active_hint = Some(minecore::solver::Explanation {
                        conclusion: minecore::solver::Deduction::Safe(1, 1),
                        deduction_type: minecore::solver::DeductionType::SubsetSafe,
                        applicable_clues: vec![(1, 0), (2, 0)],
                        relevant_unknowns: vec![(1, 1)],
                        explanation_text: "1-1 Reduction Pattern: Cell (1, 1) is guaranteed safe by subset constraint between adjacent clues.".to_string(),
                    });
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn show_main_menu(&mut self, ui: &mut egui::Ui) {
        let time = ui.input(|i| i.time);
        ui.ctx().request_repaint();

        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let accent = self.theme.colors.accent();
        let danger = self.theme.colors.danger();

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
                egui::Stroke::new(
                    1.5,
                    egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 35),
                ),
            ));
        }

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.12);

            ui.label(
                egui::RichText::new("MINE//SHIFT")
                    .size(80.0)
                    .strong()
                    .extra_letter_spacing(4.0)
                    .color(text_primary),
            );

            ui.add_space(48.0);

            let play_btn = egui::Button::new(
                egui::RichText::new("PLAY")
                    .size(20.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .fill(accent)
            .corner_radius(10.0)
            .min_size(egui::vec2(400.0, 54.0));

            if ui.add(play_btn).clicked() {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::GameModes;
            }

            ui.add_space(12.0);

            let customize_btn = egui::Button::new(
                egui::RichText::new("CUSTOMIZE")
                    .size(20.0)
                    .strong()
                    .color(accent),
            )
            .fill(card_bg)
            .stroke(egui::Stroke::new(1.5, accent))
            .corner_radius(10.0)
            .min_size(egui::vec2(400.0, 54.0));

            if ui.add(customize_btn).clicked() {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::ThemeStudio;
            }

            ui.add_space(12.0);

            let chal_btn = egui::Button::new(
                egui::RichText::new("🏆 MASTERY CHALLENGES")
                    .size(19.0)
                    .strong()
                    .color(accent),
            )
            .fill(card_bg)
            .stroke(egui::Stroke::new(1.5, accent))
            .corner_radius(10.0)
            .min_size(egui::vec2(400.0, 52.0));

            if ui.add(chal_btn).clicked() {
                self.audio.play(SoundEffect::UiClick);
                if let (Some(prof), Some(db)) = (&self.active_profile, &self.db) {
                    if let Ok(comps) = db.get_challenge_completions(prof.id) {
                        self.challenge_completions = comps;
                    }
                }
                self.screen = AppScreen::MasteryChallenges;
            }

            ui.add_space(12.0);

            let menu_btn = |ui: &mut egui::Ui, text: &str, color: egui::Color32| {
                let btn =
                    egui::Button::new(egui::RichText::new(text).size(18.0).strong().color(color))
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(10.0)
                        .min_size(egui::vec2(400.0, 48.0));
                ui.add(btn).clicked()
            };

            if menu_btn(ui, "PROFILES", text_primary) {
                self.audio.play(SoundEffect::UiClick);
                if let Some(ref d) = self.db {
                    self.profiles = d.get_profiles().unwrap_or_default();
                }
                self.screen = AppScreen::Profiles;
            }
            ui.add_space(10.0);

            if menu_btn(ui, "STATISTICS", text_primary) {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::Statistics;
            }
            ui.add_space(10.0);

            if menu_btn(ui, "ACHIEVEMENTS", text_primary) {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::Achievements;
            }
            ui.add_space(10.0);

            if menu_btn(ui, "SETTINGS", text_primary) {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::Settings;
            }
            ui.add_space(10.0);

            if menu_btn(ui, "EXIT", danger) {
                self.audio.play(SoundEffect::UiClick);
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // Show Active Profile Info at Top Right
        if let Some(ref profile) = self.active_profile {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.add_space(16.0);
                egui::Frame::new()
                    .fill(card_bg)
                    .corner_radius(10.0)
                    .stroke(egui::Stroke::new(1.0, border))
                    .inner_margin(egui::vec2(14.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&profile.avatar_id).size(22.0));
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(&profile.display_name)
                                    .size(18.0)
                                    .strong()
                                    .color(text_primary),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("Lvl {}", profile.current_level))
                                    .size(16.0)
                                    .color(accent),
                            );
                        });
                    });
            });
        }
    }

    fn show_gameplay_header(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();
        let danger = self.theme.colors.danger();
        let success = self.theme.colors.success();

        ui_tokens::card_frame(card_bg, border, 12.0, 12.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                let menu_btn = egui::Button::new(
                    egui::RichText::new("< Menu")
                        .size(16.0)
                        .strong()
                        .color(text_primary),
                )
                .fill(card_bg)
                .stroke(egui::Stroke::new(1.0, border))
                .corner_radius(8.0)
                .min_size(egui::vec2(85.0, 36.0));

                if ui.add(menu_btn).clicked() {
                    self.audio.play(SoundEffect::UiClick);
                    self.screen = AppScreen::MainMenu;
                }

                ui.add_space(12.0);

                let mode_text = match self.board.geometry {
                    minecore::board::Geometry::Square => {
                        format!("SQUARE {}×{}", self.board.width, self.board.height)
                    }
                    minecore::board::Geometry::Hex => format!("HEXAGON R{}", self.board.radius),
                };

                // Mode Pill Badge
                egui::Frame::new()
                    .fill(card_bg)
                    .corner_radius(6.0)
                    .stroke(egui::Stroke::new(1.0, border))
                    .inner_margin(egui::vec2(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(mode_text)
                                .size(13.0)
                                .strong()
                                .color(text_muted),
                        );
                    });

                // Active Challenge Badge
                if let Some(ref chal_id) = self.active_challenge_id {
                    let title = MasteryChallenge::catalog()
                        .into_iter()
                        .find(|c| c.id == *chal_id)
                        .map(|c| c.title)
                        .unwrap_or_else(|| "Challenge".to_string());
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(accent.linear_multiply(0.2))
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, accent))
                        .inner_margin(egui::vec2(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("🏆 {}", title))
                                    .size(13.0)
                                    .strong()
                                    .color(accent),
                            );
                        });
                }

                // Active Modifier Badges
                match self.board.ruleset.flag_policy {
                    FlagPolicy::NoFlags => {
                        ui.add_space(8.0);
                        egui::Frame::new()
                            .fill(danger.linear_multiply(0.2))
                            .corner_radius(6.0)
                            .stroke(egui::Stroke::new(1.0, danger))
                            .inner_margin(egui::vec2(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("🚫 NO FLAGS")
                                        .size(12.0)
                                        .strong()
                                        .color(danger),
                                );
                            });
                    }
                    FlagPolicy::LimitedFlags { max_flags } => {
                        ui.add_space(8.0);
                        egui::Frame::new()
                            .fill(card_bg)
                            .corner_radius(6.0)
                            .stroke(egui::Stroke::new(1.0, border))
                            .inner_margin(egui::vec2(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "🚩 LIMITED: {}/{}",
                                        self.board.flags_placed, max_flags
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(accent),
                                );
                            });
                    }
                    FlagPolicy::Standard => {}
                }

                if self.board.ruleset.clue_policy == CluePolicy::ExtendedRing2 {
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(card_bg)
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(140, 100, 240),
                        ))
                        .inner_margin(egui::vec2(8.0, 5.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("🔵 RING-2 CLUES")
                                    .size(12.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(160, 110, 255)),
                            );
                        });
                }

                if self.board.is_no_guess {
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(card_bg)
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, success))
                        .inner_margin(egui::vec2(8.0, 5.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("✓ NO GUESS")
                                    .size(12.0)
                                    .strong()
                                    .color(success),
                            );
                        });
                }

                if self.blitz_combo > 1 {
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(accent.linear_multiply(0.25))
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, accent))
                        .inner_margin(egui::vec2(8.0, 5.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("🔥 ×{} COMBO", self.blitz_combo))
                                    .size(12.0)
                                    .strong()
                                    .color(accent),
                            );
                        });
                }

                ui.add_space(16.0);

                let elapsed = if let Some(end) = self.board.end_time {
                    if let Some(start) = self.board.start_time {
                        end.duration_since(start).as_secs_f64()
                    } else {
                        0.0
                    }
                } else if let Some(start) = self.board.start_time {
                    std::time::Instant::now()
                        .duration_since(start)
                        .as_secs_f64()
                } else {
                    0.0
                };

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

                ui.label(
                    egui::RichText::new(time_display)
                        .size(22.0)
                        .strong()
                        .color(text_primary),
                );

                ui.add_space(16.0);

                if matches!(
                    self.mode_state.id,
                    mode::GameModeId::Blitz | mode::GameModeId::Sandbox
                ) {
                    ui.label(
                        egui::RichText::new(format!("⭐ {}", self.mode_state.score))
                            .size(22.0)
                            .strong()
                            .color(accent),
                    );
                    ui.add_space(16.0);
                }

                if matches!(
                    self.mode_state.id,
                    mode::GameModeId::Survival | mode::GameModeId::Expedition
                ) {
                    ui.label(
                        egui::RichText::new(format!("❤️ {}", self.mode_state.lives))
                            .size(22.0)
                            .strong()
                            .color(danger),
                    );
                    ui.add_space(16.0);
                }

                let mines_left = self.board.mine_count as isize - self.board.flags_placed as isize;
                ui.label(
                    egui::RichText::new(format!("🚩 {}", mines_left))
                        .size(22.0)
                        .strong()
                        .color(danger),
                );

                ui.add_space(16.0);

                let (status_label, status_bg, status_color) = match self.board.status {
                    GameStatus::Playing => ("PLAYING", card_bg, text_muted),
                    GameStatus::Won => ("VICTORY", success, egui::Color32::WHITE),
                    GameStatus::Lost => ("DEFEAT", danger, egui::Color32::WHITE),
                };

                egui::Frame::new()
                    .fill(status_bg)
                    .corner_radius(6.0)
                    .stroke(if self.board.status == GameStatus::Playing {
                        egui::Stroke::new(1.0, border)
                    } else {
                        egui::Stroke::NONE
                    })
                    .inner_margin(egui::vec2(12.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(status_label)
                                .size(14.0)
                                .strong()
                                .color(status_color),
                        );
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let restart_btn = egui::Button::new(
                        egui::RichText::new("↻ Restart")
                            .size(15.0)
                            .strong()
                            .color(text_primary),
                    )
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(8.0)
                    .min_size(egui::vec2(90.0, 36.0));

                    if ui.add(restart_btn).clicked() {
                        self.audio.play(SoundEffect::UiClick);
                        self.anim_tracker.clear();
                        self.active_hint = None;
                        self.blitz_combo = 0;
                        if let Some(ref chal_id) = self.active_challenge_id {
                            if let Some(ch) = MasteryChallenge::catalog()
                                .into_iter()
                                .find(|c| c.id == *chal_id)
                            {
                                if let Some(ref prof) = self.active_profile {
                                    if let Some(ref db) = self.db {
                                        let _ = db.record_challenge_attempt(prof.id, chal_id);
                                    }
                                }
                                self.board = Board::from_ruleset(ch.ruleset.clone());
                                if let Some(t) = ch.target_time_sec {
                                    self.mode_state.time_limit_sec = Some(t);
                                }
                            }
                        } else {
                            self.board = match self.board.geometry {
                                minecore::board::Geometry::Square => Board::new_square(
                                    self.board.width,
                                    self.board.height,
                                    self.board.mine_count,
                                ),
                                minecore::board::Geometry::Hex => {
                                    Board::new_hex(self.board.radius, self.board.mine_count)
                                }
                            };
                            self.board.is_no_guess =
                                self.mode_state.id == mode::GameModeId::NoGuess;
                        }
                    }

                    if (self.mode_state.id == mode::GameModeId::Zen
                        || self.mode_state.id == mode::GameModeId::Sandbox
                        || self.board.ruleset.assist_policy != AssistPolicy::Disallowed)
                        && self.board.status == GameStatus::Playing
                    {
                        let hint_btn = egui::Button::new(
                            egui::RichText::new("💡 Hint").size(15.0).color(accent),
                        )
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(8.0)
                        .min_size(egui::vec2(80.0, 36.0));

                        if ui.add(hint_btn).clicked() {
                            self.audio.play(SoundEffect::UiClick);
                            if let Some(exp) = Solver::explain_step(&self.board) {
                                self.active_hint = Some(exp);
                            }
                        }
                    }
                });
            });
        });

        if let Some(ref hint) = self.active_hint {
            ui.add_space(8.0);
            let deduction_label = match hint.deduction_type {
                DeductionType::DirectSafe => "Direct Safe",
                DeductionType::DirectMine => "Direct Mine",
                DeductionType::SubsetSafe => "Subset Safe",
                DeductionType::SubsetMine => "Subset Mine",
                DeductionType::Ring2Constraint => "Ring-2 Constraint",
                DeductionType::OverlappingConstraint => "Overlapping Constraint",
            };
            let is_mine = !hint.conclusion.is_safe();
            let coords = hint.conclusion.coords();
            let badge_color = if is_mine { danger } else { success };

            egui::Frame::new()
                .fill(card_bg)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.5, accent))
                .inner_margin(egui::vec2(14.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("💡 LOGIC EXPLANATION")
                                .size(14.0)
                                .strong()
                                .color(accent),
                        );
                        ui.add_space(6.0);
                        egui::Frame::new()
                            .fill(badge_color)
                            .corner_radius(4.0)
                            .inner_margin(egui::vec2(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(deduction_label)
                                        .size(11.0)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                );
                            });
                        ui.add_space(6.0);
                        let target_text = format!(
                            "Cell ({}, {}) is {}",
                            coords.0,
                            coords.1,
                            if is_mine { "a MINE 🚩" } else { "SAFE 🟢" }
                        );
                        ui.label(
                            egui::RichText::new(target_text)
                                .size(14.0)
                                .strong()
                                .color(text_primary),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!("— {}", hint.explanation_text))
                                .size(13.0)
                                .color(text_muted),
                        );
                    });
                });
        }
    }

    fn show_gameplay(&mut self, ui: &mut egui::Ui) {
        self.show_gameplay_header(ui);
        ui.add_space(20.0);

        ui.vertical_centered(|ui| {
            let avail = ui.available_size();
            let base_cell_size = 32.0;
            let base_spacing = 2.0;

            let raw_w = match self.board.geometry {
                minecore::board::Geometry::Square => {
                    (base_cell_size + base_spacing) * self.board.width as f32
                }
                minecore::board::Geometry::Hex => {
                    (base_cell_size * 3.0_f32.sqrt() + base_spacing) * self.board.width as f32
                }
            };
            let raw_h = match self.board.geometry {
                minecore::board::Geometry::Square => {
                    (base_cell_size + base_spacing) * self.board.height as f32
                }
                minecore::board::Geometry::Hex => {
                    (base_cell_size * 1.5 + base_spacing) * self.board.height as f32
                }
            };

            let scale = (avail.x / raw_w).min(avail.y / raw_h).min(1.5);
            let cell_size = base_cell_size * scale;
            let spacing = base_spacing * scale;

            let board_w = raw_w * scale;
            let board_h = raw_h * scale;

            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(board_w, board_h), egui::Sense::click());

            let pointer_pos = ui.input(|i| i.pointer.hover_pos());
            let mut clicked_cell: Option<((usize, usize), egui::Pos2)> = None;
            let mut secondary_clicked_cell: Option<((usize, usize), egui::Pos2)> = None;
            let mut middle_clicked_cell: Option<((usize, usize), egui::Pos2)> = None;
            let mut hovered_cell = None;

            // Find hovered cell using accurate center calculation for both Square and Hexagon geometries
            let mut closest_dist = f32::MAX;

            for y in 0..self.board.height {
                for x in 0..self.board.width {
                    let cell = self.board.get_cell(x, y);
                    if cell.is_none() {
                        continue;
                    }
                    let cell = cell.unwrap();
                    if !cell.active {
                        continue;
                    }

                    let time = ui.input(|i| i.time);
                    let shake_offset = self.shake.get_offset(time);
                    let center = coords::compute_cell_screen_center(
                        self.board.geometry,
                        x,
                        y,
                        self.board.radius,
                        cell_size,
                        spacing,
                        rect.min,
                        board_w,
                        board_h,
                        shake_offset,
                    );
                    let bounce = self.anim_tracker.get_bounce_scale(
                        x,
                        y,
                        time,
                        self.settings.animation_speed,
                    );
                    let draw_cell_size = cell_size * bounce;

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
                            if response.clicked() {
                                clicked_cell = Some(((x, y), center));
                            }
                            if response.secondary_clicked() {
                                secondary_clicked_cell = Some(((x, y), center));
                            }
                            if response.middle_clicked() {
                                middle_clicked_cell = Some(((x, y), center));
                            }
                        }
                    }

                    let is_hovered = hovered_cell == Some((x, y));
                    let hover_t = ui
                        .ctx()
                        .animate_bool(egui::Id::new(("hover", x, y)), is_hovered);
                    let reveal_t = ui.ctx().animate_bool(
                        egui::Id::new(("reveal", x, y)),
                        cell.state == CellState::Revealed,
                    );

                    let covered_base = self.theme.colors.covered();
                    let covered_hov = self.theme.colors.button_bg();
                    let covered_col = lerp_color(covered_base, covered_hov, hover_t);

                    let revealed_base = if cell.is_mine {
                        self.theme.colors.danger()
                    } else {
                        self.theme.colors.revealed()
                    };

                    let fill_color = lerp_color(covered_col, revealed_base, reveal_t);
                    let charcoal = self.theme.colors.text_primary();

                    match self.board.geometry {
                        minecore::board::Geometry::Square => {
                            let cell_rect = egui::Rect::from_center_size(
                                center,
                                egui::vec2(draw_cell_size, draw_cell_size),
                            );
                            let shadow_rect = cell_rect.translate(egui::vec2(0.0, 2.0));
                            ui.painter().rect_filled(
                                shadow_rect,
                                self.theme.geometry.square_radius,
                                egui::Color32::from_black_alpha(20),
                            );
                            ui.painter().rect_filled(
                                cell_rect,
                                self.theme.geometry.square_radius,
                                fill_color,
                            );
                            if self.theme.geometry.border_thickness > 0.0 {
                                ui.painter().rect_stroke(
                                    cell_rect,
                                    self.theme.geometry.square_radius,
                                    egui::Stroke::new(
                                        self.theme.geometry.border_thickness,
                                        self.theme.colors.border(),
                                    ),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            if cell.clue_type == minecore::board::ClueType::Ring2 {
                                let inner_rect = cell_rect.shrink(3.0);
                                ui.painter().rect_stroke(
                                    inner_rect,
                                    self.theme.geometry.square_radius.max(2.0),
                                    egui::Stroke::new(1.5, egui::Color32::from_rgb(140, 100, 240)),
                                    egui::StrokeKind::Inside,
                                );
                            }
                        }
                        minecore::board::Geometry::Hex => {
                            let points = coords::hex_vertices(center, draw_cell_size * 0.95);
                            let shadow_points: Vec<egui::Pos2> =
                                points.iter().map(|p| *p + egui::vec2(0.0, 2.0)).collect();
                            ui.painter().add(egui::Shape::convex_polygon(
                                shadow_points,
                                egui::Color32::from_black_alpha(20),
                                egui::Stroke::NONE,
                            ));
                            ui.painter().add(egui::Shape::convex_polygon(
                                points.clone(),
                                fill_color,
                                egui::Stroke::NONE,
                            ));
                            if self.theme.geometry.border_thickness > 0.0 {
                                ui.painter().add(egui::Shape::closed_line(
                                    points.clone(),
                                    egui::Stroke::new(
                                        self.theme.geometry.border_thickness,
                                        self.theme.colors.border(),
                                    ),
                                ));
                            }
                            if cell.clue_type == minecore::board::ClueType::Ring2 {
                                let inner_points =
                                    coords::hex_vertices(center, draw_cell_size * 0.78);
                                ui.painter().add(egui::Shape::closed_line(
                                    inner_points,
                                    egui::Stroke::new(1.5, egui::Color32::from_rgb(140, 100, 240)),
                                ));
                            }
                        }
                    }

                    match cell.state {
                        CellState::Flagged => {
                            let base_y = center.y + cell_size * 0.2;
                            let tip_y = center.y - cell_size * 0.25;
                            let flag_right = center.x + cell_size * 0.2;
                            ui.painter().line_segment(
                                [egui::pos2(center.x, base_y), egui::pos2(center.x, tip_y)],
                                egui::Stroke::new(2.0, charcoal),
                            );
                            ui.painter().add(egui::Shape::convex_polygon(
                                vec![
                                    egui::pos2(center.x, tip_y),
                                    egui::pos2(flag_right, tip_y + cell_size * 0.1),
                                    egui::pos2(center.x, tip_y + cell_size * 0.2),
                                ],
                                self.theme.colors.danger(),
                                egui::Stroke::NONE,
                            ));
                        }
                        CellState::Revealed => {
                            if cell.is_mine {
                                ui.painter()
                                    .circle_filled(center, cell_size * 0.25, charcoal);
                                for i in 0..8 {
                                    let a = (i as f32 * 45.0).to_radians();
                                    let r = cell_size * 0.35;
                                    ui.painter().line_segment(
                                        [center, center + egui::vec2(a.cos() * r, a.sin() * r)],
                                        egui::Stroke::new(2.0, charcoal),
                                    );
                                }
                                ui.painter().circle_filled(
                                    center + egui::vec2(-cell_size * 0.08, -cell_size * 0.08),
                                    cell_size * 0.08,
                                    egui::Color32::WHITE,
                                );
                            } else if cell.adjacent_mines > 0
                                || (cell.clue_type == minecore::board::ClueType::Ring2
                                    && cell.ring2_mines > 0)
                            {
                                let (clue_val, text_color) =
                                    if cell.clue_type == minecore::board::ClueType::Ring2 {
                                        (cell.ring2_mines, egui::Color32::from_rgb(160, 110, 255))
                                    } else {
                                        (
                                            cell.adjacent_mines,
                                            self.theme
                                                .colors
                                                .number_color(cell.adjacent_mines as usize),
                                        )
                                    };
                                ui.painter().text(
                                    center,
                                    egui::Align2::CENTER_CENTER,
                                    clue_val.to_string(),
                                    egui::FontId::proportional(cell_size * 0.7),
                                    text_color,
                                );
                                if cell.clue_type == minecore::board::ClueType::Ring2 {
                                    ui.painter().circle_stroke(
                                        center + egui::vec2(cell_size * 0.28, -cell_size * 0.28),
                                        cell_size * 0.08,
                                        egui::Stroke::new(
                                            1.5,
                                            egui::Color32::from_rgb(140, 100, 240),
                                        ),
                                    );
                                }
                            }
                        }
                        CellState::Covered => {}
                    }

                    if let Some(ref hint) = self.active_hint {
                        if hint.conclusion.coords() == (x, y) {
                            let pulse = (time * 4.0).sin() * 0.5 + 0.5;
                            let halo_r = draw_cell_size * (0.6 + 0.15 * pulse as f32);
                            let halo_col = if !hint.conclusion.is_safe() {
                                egui::Color32::from_rgba_unmultiplied(
                                    230,
                                    76,
                                    60,
                                    (140.0 + 80.0 * pulse) as u8,
                                )
                            } else {
                                egui::Color32::from_rgba_unmultiplied(
                                    80,
                                    200,
                                    120,
                                    (140.0 + 80.0 * pulse) as u8,
                                )
                            };
                            ui.painter().circle_stroke(
                                center,
                                halo_r,
                                egui::Stroke::new(3.0, halo_col),
                            );
                        }
                    }
                }
            }

            let old_status = self.board.status;
            let time = ui.input(|i| i.time);
            if clicked_cell.is_some()
                || secondary_clicked_cell.is_some()
                || middle_clicked_cell.is_some()
            {
                self.active_hint = None;
            }
            if let Some(((x, y), pt)) = clicked_cell {
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
                    let was_covered = self
                        .board
                        .get_cell(x, y)
                        .map(|c| c.state == CellState::Covered)
                        .unwrap_or(false);
                    if was_covered {
                        self.audio.play(SoundEffect::TileClick);
                        self.anim_tracker.on_reveal(x, y, time);
                        self.particles.spawn_tile_pop(
                            pt,
                            self.theme.colors.accent(),
                            self.settings.cursor.reduced_motion,
                        );
                        self.shake.add_trauma(0.04);

                        if self.mode_state.id == mode::GameModeId::Blitz {
                            if time - self.blitz_last_reveal_time < 2.0 {
                                self.blitz_combo = (self.blitz_combo + 1).min(5);
                            } else {
                                self.blitz_combo = 1;
                            }
                            self.blitz_last_reveal_time = time;
                            self.mode_state.score += (20 * self.blitz_combo) as i64;
                        }
                    }
                    let was_covered_cells: Vec<(usize, usize)> = (0..self.board.height)
                        .flat_map(|cy| (0..self.board.width).map(move |cx| (cx, cy)))
                        .filter(|&(cx, cy)| {
                            self.board
                                .get_cell(cx, cy)
                                .map_or(false, |c| c.state == CellState::Covered)
                        })
                        .collect();

                    self.board.reveal(x, y);

                    for (cx, cy) in was_covered_cells {
                        if let Some(c) = self.board.get_cell(cx, cy) {
                            if c.state == CellState::Revealed {
                                self.anim_tracker.on_reveal(cx, cy, time);
                            }
                        }
                    }

                    if self.board.status == GameStatus::Lost {
                        if self.mode_state.id == mode::GameModeId::Survival
                            && self.mode_state.lives > 1
                        {
                            self.mode_state.lives -= 1;
                            self.board.status = GameStatus::Playing;
                            let idx = self.board.get_index(x, y);
                            self.board.cells[idx].state = CellState::Flagged;
                            self.board.flags_placed += 1;
                            self.audio.play(SoundEffect::Explosion);
                            self.shake.add_trauma(0.5);
                            self.particles.spawn_explosion(
                                pt,
                                self.theme.colors.danger(),
                                self.settings.cursor.reduced_motion,
                            );
                        } else {
                            self.audio.play(SoundEffect::Explosion);
                            self.audio.play(SoundEffect::Defeat);
                            self.shake.add_trauma(0.85);
                            self.particles.spawn_explosion(
                                pt,
                                self.theme.colors.danger(),
                                self.settings.cursor.reduced_motion,
                            );
                        }
                    } else if self.board.status == GameStatus::Won {
                        self.audio.play(SoundEffect::Victory);
                        self.particles
                            .spawn_victory(rect, 80, self.settings.cursor.reduced_motion);
                        self.shake.add_trauma(0.2);
                    } else if let Some(c) = self.board.get_cell(x, y) {
                        if c.state == CellState::Revealed {
                            self.audio.play(SoundEffect::TileReveal(c.adjacent_mines));
                        }
                    }
                }
            }
            if let Some(((x, y), pt)) = secondary_clicked_cell {
                let was_flagged = self
                    .board
                    .get_cell(x, y)
                    .map(|c| c.state == CellState::Flagged)
                    .unwrap_or(false);
                self.board.toggle_flag(x, y);
                if !was_flagged {
                    self.audio.play(SoundEffect::FlagPlace);
                    self.particles.spawn_tile_pop(
                        pt,
                        self.theme.colors.danger(),
                        self.settings.cursor.reduced_motion,
                    );
                    self.shake.add_trauma(0.04);
                } else {
                    self.audio.play(SoundEffect::FlagRemove);
                }
            }
            if let Some(((x, y), pt)) = middle_clicked_cell {
                self.audio.play(SoundEffect::Chord);
                self.shake.add_trauma(0.12);
                self.particles.spawn_tile_pop(
                    pt,
                    self.theme.colors.accent(),
                    self.settings.cursor.reduced_motion,
                );

                let was_covered_cells: Vec<(usize, usize)> = (0..self.board.height)
                    .flat_map(|cy| (0..self.board.width).map(move |cx| (cx, cy)))
                    .filter(|&(cx, cy)| {
                        self.board
                            .get_cell(cx, cy)
                            .map_or(false, |c| c.state == CellState::Covered)
                    })
                    .collect();

                self.board.chord(x, y);

                for (cx, cy) in was_covered_cells {
                    if let Some(c) = self.board.get_cell(cx, cy) {
                        if c.state == CellState::Revealed {
                            self.anim_tracker.on_reveal(cx, cy, time);
                        }
                    }
                }

                if self.board.status == GameStatus::Lost {
                    self.audio.play(SoundEffect::Explosion);
                    self.audio.play(SoundEffect::Defeat);
                    self.shake.add_trauma(0.85);
                    self.particles.spawn_explosion(
                        pt,
                        self.theme.colors.danger(),
                        self.settings.cursor.reduced_motion,
                    );
                } else if self.board.status == GameStatus::Won {
                    self.audio.play(SoundEffect::Victory);
                    self.particles
                        .spawn_victory(rect, 80, self.settings.cursor.reduced_motion);
                    self.shake.add_trauma(0.2);
                }
            }

            if old_status == GameStatus::Playing && self.board.status != GameStatus::Playing {
                let mut true_end = true;

                match self.mode_state.id {
                    mode::GameModeId::Blitz => {
                        if self.board.status == GameStatus::Won {
                            self.mode_state.score += 500;
                        } else {
                            self.mode_state.score = (self.mode_state.score - 150).max(0);
                        }
                        if self.mode_state.time_limit_sec.unwrap_or(0.0)
                            - self.board.start_time.unwrap().elapsed().as_secs_f64()
                            > 0.0
                        {
                            // Instant restart for Blitz
                            self.board = match self.board.geometry {
                                minecore::board::Geometry::Square => Board::new_square(
                                    self.board.width,
                                    self.board.height,
                                    self.board.mine_count,
                                ),
                                minecore::board::Geometry::Hex => {
                                    Board::new_hex(self.board.radius, self.board.mine_count)
                                }
                            };
                            true_end = false;
                        }
                    }
                    mode::GameModeId::Survival => {
                        if self.board.status == GameStatus::Won {
                            self.mode_state.stage += 1;
                            self.board = match self.board.geometry {
                                minecore::board::Geometry::Square => Board::new_square(
                                    self.board.width,
                                    self.board.height,
                                    self.board.mine_count + 2,
                                ),
                                minecore::board::Geometry::Hex => {
                                    Board::new_hex(self.board.radius, self.board.mine_count + 2)
                                }
                            };
                            true_end = false;
                        } else {
                            self.mode_state.lives -= 1;
                            if self.mode_state.lives > 0 {
                                self.board = match self.board.geometry {
                                    minecore::board::Geometry::Square => Board::new_square(
                                        self.board.width,
                                        self.board.height,
                                        self.board.mine_count,
                                    ),
                                    minecore::board::Geometry::Hex => {
                                        Board::new_hex(self.board.radius, self.board.mine_count)
                                    }
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
                                    minecore::board::Geometry::Square => Board::new_square(
                                        self.board.width,
                                        self.board.height,
                                        (20 + self.mode_state.stage).min(60) as usize,
                                    ),
                                    minecore::board::Geometry::Hex => Board::new_hex(
                                        self.board.radius,
                                        (20 + self.mode_state.stage).min(60) as usize,
                                    ),
                                };
                                self.board.seed = Some(self.mode_state.stage as u64);
                                if let (Some(db), Some(profile)) = (&self.db, &self.active_profile)
                                {
                                    // Ensure we don't regress if they replay a lower stage
                                    let current = db.get_stat(profile.id, "expedition_stage");
                                    if self.mode_state.stage as i64 > current {
                                        let _ = db.set_highest_stat(
                                            profile.id,
                                            "expedition_stage",
                                            self.mode_state.stage as i64,
                                        );
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
                        let duration_ms = if let (Some(s), Some(e)) =
                            (self.board.start_time, self.board.end_time)
                        {
                            e.duration_since(s).as_millis() as i64
                        } else {
                            0
                        };
                        let started_at = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64
                            - (duration_ms / 1000);

                        let session = db::GameSession {
                            profile_id: profile.id,
                            mode: self.mode_state.id.name().to_string(),
                            geometry: match self.board.geometry {
                                minecore::board::Geometry::Square => "Square".to_string(),
                                minecore::board::Geometry::Hex => "Hex".to_string(),
                            },
                            board_width: self.board.width as i64,
                            board_height: self.board.height as i64,
                            hex_radius: self.board.radius as i64,
                            mine_count: self.board.mine_count as i64,
                            started_at,
                            duration_ms,
                            result: match self.board.status {
                                GameStatus::Won => "Won".to_string(),
                                _ => "Lost".to_string(),
                            },
                        };
                        let _ = db.save_session(&session);
                        let _ = db.add_stat(profile.id, "total_playtime_s", duration_ms / 1000);

                        if self.board.status == GameStatus::Won {
                            let _ = db.add_xp(profile.id, 50);
                            if self.board.geometry == minecore::board::Geometry::Square {
                                let _ = db.set_best_time(
                                    profile.id,
                                    "best_time_square",
                                    duration_ms / 1000,
                                );
                                let _ = db.unlock_achievement(profile.id, "first_win");
                                if self.board.mine_count >= 99 {
                                    let _ = db.unlock_achievement(profile.id, "expert_clear");
                                }
                            } else {
                                let _ = db.set_best_time(
                                    profile.id,
                                    "best_time_hex",
                                    duration_ms / 1000,
                                );
                                let _ = db.unlock_achievement(profile.id, "hex_master");
                            }

                            if self.mode_state.id == mode::GameModeId::Blitz
                                && self.mode_state.score >= 2000
                            {
                                let _ = db.unlock_achievement(profile.id, "blitz_star");
                            }
                            if self.mode_state.id == mode::GameModeId::Survival
                                && self.mode_state.stage >= 5
                            {
                                let _ = db.unlock_achievement(profile.id, "survival_5");
                            }
                            if self.mode_state.id == mode::GameModeId::Expedition
                                && self.mode_state.stage >= 10
                            {
                                let _ = db.unlock_achievement(profile.id, "expedition_10");
                            }
                            if self.mode_state.id == mode::GameModeId::Daily {
                                let _ = db.unlock_achievement(profile.id, "daily_win");
                            }

                            if let Some(ref chal_id) = self.active_challenge_id {
                                if let Some(ch) = MasteryChallenge::catalog()
                                    .into_iter()
                                    .find(|c| c.id == *chal_id)
                                {
                                    let _ = db.record_challenge_completion(
                                        profile.id,
                                        chal_id,
                                        duration_ms,
                                        ch.xp_reward,
                                    );
                                    if let Ok(comps) = db.get_challenge_completions(profile.id) {
                                        self.challenge_completions = comps;
                                    }
                                }
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
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let _text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        if ui_tokens::screen_header(ui, "PLAYER PROFILES", text_primary, border, card_bg, |ui| {
            if ui
                .button(
                    egui::RichText::new("✨ Cosmetics")
                        .size(15.0)
                        .strong()
                        .color(text_primary),
                )
                .clicked()
            {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::ProfileCosmetics;
            }
        }) {
            if self.active_profile.is_some() {
                self.audio.play(SoundEffect::UiClick);
                self.screen = AppScreen::MainMenu;
            }
        }

        ui.add_space(16.0);

        ui_tokens::centered_container(ui, 820.0, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let mut action: Option<(&str, db::Profile)> = None;

                    for profile in &self.profiles {
                        let is_active = self
                            .active_profile
                            .as_ref()
                            .map_or(false, |p| p.id == profile.id);
                        let frame_stroke = if is_active {
                            egui::Stroke::new(2.0, accent)
                        } else {
                            egui::Stroke::new(1.0, border)
                        };

                        egui::Frame::new()
                            .fill(card_bg)
                            .corner_radius(12.0)
                            .stroke(frame_stroke)
                            .inner_margin(18.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&profile.avatar_id).size(42.0));
                                    ui.add_space(10.0);
                                    ui.vertical(|ui| {
                                        if Some(profile.id) == self.rename_profile_id {
                                            ui.horizontal(|ui| {
                                                ui.text_edit_singleline(&mut self.rename_buffer);
                                                if ui.button("Save").clicked() {
                                                    action = Some(("rename_save", profile.clone()));
                                                }
                                                if ui.button("Cancel").clicked() {
                                                    action =
                                                        Some(("rename_cancel", profile.clone()));
                                                }
                                            });
                                        } else {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&profile.display_name)
                                                        .size(22.0)
                                                        .strong()
                                                        .color(text_primary),
                                                );
                                                if is_active {
                                                    ui.add_space(8.0);
                                                    egui::Frame::new()
                                                        .fill(accent)
                                                        .corner_radius(12.0)
                                                        .inner_margin(egui::vec2(10.0, 3.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new("ACTIVE")
                                                                    .size(12.0)
                                                                    .strong()
                                                                    .color(egui::Color32::WHITE),
                                                            );
                                                        });
                                                }
                                            });
                                        }

                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Level {}",
                                                profile.current_level
                                            ))
                                            .size(15.0)
                                            .strong()
                                            .color(accent),
                                        );

                                        let mut req = 100;
                                        let mut lvl = 1;
                                        let mut prev_req = 0;
                                        while lvl < profile.current_level {
                                            prev_req = req;
                                            lvl += 1;
                                            req += lvl * 100;
                                        }
                                        let progress = (profile.total_xp as f32 - prev_req as f32)
                                            / (req as f32 - prev_req as f32).max(1.0);

                                        let pb = egui::ProgressBar::new(progress)
                                            .text(format!("{} / {} XP", profile.total_xp, req))
                                            .fill(accent);
                                        ui.add_sized([220.0, 16.0], pb);
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if !is_active {
                                                if ui
                                                    .button(
                                                        egui::RichText::new("Delete")
                                                            .size(15.0)
                                                            .color(self.theme.colors.danger()),
                                                    )
                                                    .clicked()
                                                {
                                                    action = Some(("delete", profile.clone()));
                                                }
                                                ui.add_space(8.0);
                                                if ui
                                                    .button(
                                                        egui::RichText::new("Rename")
                                                            .size(15.0)
                                                            .color(text_primary),
                                                    )
                                                    .clicked()
                                                {
                                                    action = Some(("rename", profile.clone()));
                                                }
                                                ui.add_space(8.0);
                                                let select_btn = egui::Button::new(
                                                    egui::RichText::new("Select")
                                                        .size(15.0)
                                                        .strong()
                                                        .color(egui::Color32::WHITE),
                                                )
                                                .fill(accent)
                                                .corner_radius(6.0);
                                                if ui.add(select_btn).clicked() {
                                                    action = Some(("select", profile.clone()));
                                                }
                                            } else {
                                                if ui
                                                    .button(
                                                        egui::RichText::new("Rename")
                                                            .size(15.0)
                                                            .color(text_primary),
                                                    )
                                                    .clicked()
                                                {
                                                    action = Some(("rename", profile.clone()));
                                                }
                                            }
                                        },
                                    );
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
                            self.load_profile_settings();
                            self.apply_profile_theme();
                            self.screen = AppScreen::MainMenu;
                        } else if cmd == "rename" {
                            self.rename_profile_id = Some(profile.id);
                            self.rename_buffer = profile.display_name.clone();
                        } else if cmd == "rename_save" {
                            if let Some(ref db) = self.db {
                                let _ = db.rename_profile(profile.id, &self.rename_buffer);
                                self.profiles = db.get_profiles().unwrap_or_default();
                                if let Some(p) = self.profiles.iter().find(|p| p.id == profile.id) {
                                    if self.active_profile.as_ref().map_or(false, |a| a.id == p.id)
                                    {
                                        self.active_profile = Some(p.clone());
                                    }
                                }
                            }
                            self.rename_profile_id = None;
                        } else if cmd == "rename_cancel" {
                            self.rename_profile_id = None;
                        }
                    }

                    ui.add_space(20.0);
                    ui_tokens::card_frame(card_bg, border, 12.0, 18.0).show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Create New Profile")
                                .size(20.0)
                                .strong()
                                .color(text_primary),
                        );
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Name:").size(16.0).color(text_primary));
                            ui.text_edit_singleline(&mut self.new_profile_name);
                            ui.add_space(8.0);
                            let create_btn = egui::Button::new(
                                egui::RichText::new("Create Profile")
                                    .size(16.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(accent)
                            .corner_radius(6.0);
                            if ui.add(create_btn).clicked() {
                                if !self.new_profile_name.trim().is_empty() {
                                    if let Some(ref db) = self.db {
                                        if let Ok(id) =
                                            db.create_profile(self.new_profile_name.trim(), "👶")
                                        {
                                            self.profiles = db.get_profiles().unwrap_or_default();
                                            if let Some(p) =
                                                self.profiles.iter().find(|p| p.id == id)
                                            {
                                                self.active_profile = Some(p.clone());
                                                self.load_profile_settings();
                                                self.apply_profile_theme();
                                                self.screen = AppScreen::MainMenu;
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    });
                    ui.add_space(30.0);
                });
        });
    }

    fn show_statistics(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        if ui_tokens::screen_header(ui, "STATISTICS", text_primary, border, card_bg, |ui| {
            if let Some(ref p) = self.active_profile {
                egui::Frame::new()
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(16.0)
                    .inner_margin(egui::vec2(14.0, 5.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} {}", p.avatar_id, p.display_name))
                                .size(14.0)
                                .strong()
                                .color(text_primary),
                        );
                    });
            }
        }) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::MainMenu;
        }

        ui.add_space(20.0);

        ui_tokens::centered_container(ui, 960.0, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if let Some(ref p) = self.active_profile {
                        if let Some(ref db) = self.db {
                            let played = db.get_stat(p.id, "games_played");
                            let won = db.get_stat(p.id, "games_won");
                            let win_rate = if played > 0 {
                                (won as f64 / played as f64 * 100.0) as i64
                            } else {
                                0
                            };
                            let best_square = db.get_stat(p.id, "best_time_square");
                            let best_hex = db.get_stat(p.id, "best_time_hex");
                            let total_playtime = db.get_stat(p.id, "total_playtime_s");
                            let expedition_stage = db.get_stat(p.id, "expedition_stage").max(1);

                            let hours = total_playtime / 3600;
                            let mins = (total_playtime % 3600) / 60;
                            let time_str = format!("{}h {}m", hours, mins);

                            ui.label(
                                egui::RichText::new("CAREER OVERVIEW")
                                    .size(14.0)
                                    .strong()
                                    .color(accent),
                            );
                            ui.add_space(8.0);

                            // Row 1: Primary Metrics (4 cards)
                            let card_w = (ui.available_width() - 36.0) / 4.0;
                            let stat_card =
                                |ui: &mut egui::Ui,
                                 title: &str,
                                 value: &str,
                                 val_color: egui::Color32| {
                                    ui_tokens::card_frame(card_bg, border, 12.0, 16.0).show(
                                        ui,
                                        |ui| {
                                            ui.set_width(card_w);
                                            ui.label(
                                                egui::RichText::new(title.to_uppercase())
                                                    .size(12.0)
                                                    .strong()
                                                    .color(text_muted),
                                            );
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(value)
                                                    .size(30.0)
                                                    .strong()
                                                    .color(val_color),
                                            );
                                        },
                                    );
                                };

                            ui.horizontal(|ui| {
                                stat_card(ui, "Games Played", &played.to_string(), text_primary);
                                ui.add_space(12.0);
                                stat_card(
                                    ui,
                                    "Games Won",
                                    &won.to_string(),
                                    self.theme.colors.success(),
                                );
                                ui.add_space(12.0);
                                stat_card(ui, "Win Rate", &format!("{}%", win_rate), accent);
                                ui.add_space(12.0);
                                stat_card(ui, "Total Playtime", &time_str, text_primary);
                            });

                            ui.add_space(20.0);
                            ui.label(
                                egui::RichText::new("RECORDS & MILESTONES")
                                    .size(14.0)
                                    .strong()
                                    .color(accent),
                            );
                            ui.add_space(8.0);

                            // Row 2: Performance Records (4 cards)
                            let sq_str = if best_square > 0 {
                                format!("{}s", best_square)
                            } else {
                                "--".to_string()
                            };
                            let hex_str = if best_hex > 0 {
                                format!("{}s", best_hex)
                            } else {
                                "--".to_string()
                            };

                            ui.horizontal(|ui| {
                                stat_card(ui, "Best Square Time", &sq_str, text_primary);
                                ui.add_space(12.0);
                                stat_card(ui, "Best Hex Time", &hex_str, text_primary);
                                ui.add_space(12.0);
                                stat_card(
                                    ui,
                                    "Expedition Stage",
                                    &format!("Stage {}", expedition_stage),
                                    accent,
                                );
                                ui.add_space(12.0);
                                stat_card(
                                    ui,
                                    "Player Level",
                                    &format!("Lv {}", p.current_level),
                                    self.theme.colors.success(),
                                );
                            });
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("Select a profile to view statistics")
                                .size(16.0)
                                .color(text_muted),
                        );
                    }
                    ui.add_space(30.0);
                });
        });
    }

    fn show_achievements(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        let achievements_list = vec![
            ("first_win", "First Victory", "Win your first game."),
            (
                "blitz_star",
                "Blitz Star",
                "Score 2000 points in Blitz mode.",
            ),
            ("survival_5", "Survivor", "Reach Stage 5 in Survival mode."),
            (
                "expedition_10",
                "Explorer",
                "Reach Stage 10 in Expedition mode.",
            ),
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
            (
                "fast_30",
                "Lightning Fast",
                "Win any game under 30 seconds.",
            ),
            (
                "fast_10",
                "Blink of an Eye",
                "Win any game under 10 seconds.",
            ),
            (
                "playtime_1h",
                "Just One More",
                "Accumulate 1 hour of playtime.",
            ),
            (
                "playtime_10h",
                "Time Well Spent",
                "Accumulate 10 hours of playtime.",
            ),
            (
                "playtime_100h",
                "Chronos",
                "Accumulate 100 hours of playtime.",
            ),
            (
                "chord_master",
                "Efficiency",
                "Win a game using chording extensively.",
            ),
            ("no_flags", "Daredevil", "Win without placing any flags."),
            ("lose_1", "First Blood", "Lose a game. It happens."),
            ("lose_100", "Stubborn", "Lose 100 games. Keep trying!"),
            (
                "first_hex",
                "New Dimensions",
                "Play your first Hexagonal game.",
            ),
            ("level_5", "Level 5", "Reach profile level 5."),
            ("level_10", "Level 10", "Reach profile level 10."),
            ("level_50", "Level 50", "Reach profile level 50."),
        ];

        let unlocked: std::collections::HashSet<String> =
            if let (Some(p), Some(db)) = (&self.active_profile, &self.db) {
                db.get_unlocked_achievements(p.id)
                    .unwrap_or_default()
                    .into_iter()
                    .collect()
            } else {
                std::collections::HashSet::new()
            };

        let total_count = achievements_list.len();
        let unlocked_count = achievements_list
            .iter()
            .filter(|(id, _, _)| unlocked.contains(&id.to_string()))
            .count();
        let progress_pct = if total_count > 0 {
            unlocked_count as f32 / total_count as f32
        } else {
            0.0
        };

        if ui_tokens::screen_header(ui, "ACHIEVEMENTS", text_primary, border, card_bg, |ui| {
            egui::Frame::new()
                .fill(card_bg)
                .stroke(egui::Stroke::new(1.0, border))
                .corner_radius(16.0)
                .inner_margin(egui::vec2(16.0, 6.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "🏆 {} / {} Unlocked ({:.0}%)",
                            unlocked_count,
                            total_count,
                            progress_pct * 100.0
                        ))
                        .size(14.0)
                        .strong()
                        .color(accent),
                    );
                });
        }) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::MainMenu;
        }

        ui.add_space(14.0);

        ui_tokens::centered_container(ui, 1020.0, |ui| {
            // Progress bar across container
            let pb = egui::ProgressBar::new(progress_pct).fill(accent);
            ui.add_sized([ui.available_width(), 12.0], pb);

            ui.add_space(14.0);

            // Filter pills: All | Unlocked | Locked
            ui.horizontal(|ui| {
                let filters = [
                    (AchievementFilter::All, format!("All ({})", total_count)),
                    (
                        AchievementFilter::Unlocked,
                        format!("Unlocked ({})", unlocked_count),
                    ),
                    (
                        AchievementFilter::Locked,
                        format!("Locked ({})", total_count - unlocked_count),
                    ),
                ];

                for (f, label) in filters {
                    let is_active = self.achievement_filter == f;
                    let btn =
                        egui::Button::new(egui::RichText::new(label).size(14.0).strong().color(
                            if is_active {
                                egui::Color32::WHITE
                            } else {
                                text_primary
                            },
                        ))
                        .fill(if is_active { accent } else { card_bg })
                        .stroke(if is_active {
                            egui::Stroke::NONE
                        } else {
                            egui::Stroke::new(1.0, border)
                        })
                        .corner_radius(14.0)
                        .min_size(egui::vec2(110.0, 32.0));

                    if ui.add(btn).clicked() {
                        self.achievement_filter = f;
                    }
                    ui.add_space(8.0);
                }
            });

            ui.add_space(16.0);

            // Responsive 2-Column Grid of Achievements (precisely fitted)
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let col_spacing = 16.0;
                    let card_w = ((ui.available_width() - col_spacing) / 2.0)
                        .floor()
                        .max(200.0);

                    let filtered_achievements: Vec<_> = achievements_list
                        .into_iter()
                        .filter(|(id, _, _)| {
                            let is_unlocked = unlocked.contains(&id.to_string());
                            match self.achievement_filter {
                                AchievementFilter::All => true,
                                AchievementFilter::Unlocked => is_unlocked,
                                AchievementFilter::Locked => !is_unlocked,
                            }
                        })
                        .collect();

                    for chunk in filtered_achievements.chunks(2) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = col_spacing;
                            for (id, title, desc) in chunk {
                                let is_unlocked = unlocked.contains(&id.to_string());
                                let frame_stroke = if is_unlocked {
                                    egui::Stroke::new(1.5, accent)
                                } else {
                                    egui::Stroke::new(1.0, border)
                                };

                                egui::Frame::new()
                                    .fill(card_bg)
                                    .stroke(frame_stroke)
                                    .corner_radius(12.0)
                                    .inner_margin(16.0)
                                    .show(ui, |ui| {
                                        ui.set_width(card_w);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(if is_unlocked {
                                                    "🏆"
                                                } else {
                                                    "🔒"
                                                })
                                                .size(36.0),
                                            );
                                            ui.add_space(10.0);
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(*title)
                                                            .size(16.0)
                                                            .strong()
                                                            .color(if is_unlocked {
                                                                text_primary
                                                            } else {
                                                                text_muted
                                                            }),
                                                    );
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            let badge_fill = if is_unlocked {
                                                                self.theme.colors.success()
                                                            } else if self.theme.is_dark() {
                                                                egui::Color32::from_white_alpha(22)
                                                            } else {
                                                                egui::Color32::from_black_alpha(26)
                                                            };
                                                            egui::Frame::new()
                                                                .fill(badge_fill)
                                                                .corner_radius(8.0)
                                                                .inner_margin(egui::vec2(8.0, 2.0))
                                                                .show(ui, |ui| {
                                                                    ui.label(
                                                                        egui::RichText::new(
                                                                            if is_unlocked {
                                                                                "UNLOCKED"
                                                                            } else {
                                                                                "LOCKED"
                                                                            },
                                                                        )
                                                                        .size(10.0)
                                                                        .strong()
                                                                        .color(if is_unlocked {
                                                                            egui::Color32::WHITE
                                                                        } else {
                                                                            text_muted
                                                                        }),
                                                                    );
                                                                });
                                                        },
                                                    );
                                                });
                                                ui.add_space(4.0);
                                                ui.label(
                                                    egui::RichText::new(*desc)
                                                        .size(13.0)
                                                        .color(text_muted),
                                                );
                                            });
                                        });
                                    });
                            }
                        });
                        ui.add_space(10.0);
                    }
                    ui.add_space(30.0);
                });
        });
    }

    fn show_game_modes(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        if ui_tokens::screen_header(ui, "GAME MODES", text_primary, border, card_bg, |ui| {
            if ui
                .button(
                    egui::RichText::new("🏆 Mastery Challenges")
                        .size(15.0)
                        .strong()
                        .color(accent),
                )
                .clicked()
            {
                self.audio.play(SoundEffect::UiClick);
                if let (Some(prof), Some(db)) = (&self.active_profile, &self.db) {
                    if let Ok(comps) = db.get_challenge_completions(prof.id) {
                        self.challenge_completions = comps;
                    }
                }
                self.screen = AppScreen::MasteryChallenges;
            }
        }) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::MainMenu;
        }

        ui.add_space(16.0);

        ui_tokens::centered_container(ui, 860.0, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
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
                        egui::Frame::new()
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(1.0, border))
                            .corner_radius(12.0)
                            .inner_margin(16.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).size(32.0));
                                    ui.add_space(12.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} - {}",
                                                id.name(),
                                                variant
                                            ))
                                            .size(19.0)
                                            .strong()
                                            .color(text_primary),
                                        );
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(id.desc())
                                                .size(14.0)
                                                .color(text_muted),
                                        );
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let play_btn = egui::Button::new(
                                                egui::RichText::new("PLAY")
                                                    .size(15.0)
                                                    .strong()
                                                    .color(egui::Color32::WHITE),
                                            )
                                            .fill(accent)
                                            .corner_radius(8.0)
                                            .min_size(egui::vec2(96.0, 38.0));

                                            if ui.add(play_btn).clicked() {
                                                self.audio.play(SoundEffect::UiClick);
                                                self.active_challenge_id = None;
                                                self.active_hint = None;
                                                self.blitz_combo = 0;

                                                if id == mode::GameModeId::Sandbox {
                                                    self.screen = AppScreen::SandboxLab;
                                                    return;
                                                }

                                                self.mode_state = mode::ModeState::new(id.clone());
                                                if variant == "SQUARE" {
                                                    // Intermediate
                                                    self.board = Board::new_square(16, 16, 40);
                                                } else {
                                                    self.board = Board::new_hex(8, 40);
                                                }

                                                if self.mode_state.id == mode::GameModeId::NoGuess {
                                                    self.board.is_no_guess = true;
                                                } else if self.mode_state.id
                                                    == mode::GameModeId::Blitz
                                                {
                                                    self.mode_state.time_limit_sec = Some(180.0);
                                                    self.mode_state.score = 0;
                                                } else if self.mode_state.id
                                                    == mode::GameModeId::Survival
                                                {
                                                    self.mode_state.lives = 3;
                                                    self.mode_state.stage = 1;
                                                } else if self.mode_state.id
                                                    == mode::GameModeId::Expedition
                                                {
                                                    self.mode_state.lives = 1;
                                                    if let (Some(db), Some(profile)) =
                                                        (&self.db, &self.active_profile)
                                                    {
                                                        self.mode_state.stage = db
                                                            .get_stat(
                                                                profile.id,
                                                                "expedition_stage",
                                                            )
                                                            .max(1)
                                                            as i32;
                                                    } else {
                                                        self.mode_state.stage = 1;
                                                    }
                                                    self.board.seed =
                                                        Some(self.mode_state.stage as u64);
                                                    self.board.mine_count =
                                                        (20 + self.mode_state.stage).min(60)
                                                            as usize;
                                                } else if self.mode_state.id
                                                    == mode::GameModeId::Daily
                                                {
                                                    use std::collections::hash_map::DefaultHasher;
                                                    use std::hash::{Hash, Hasher};
                                                    let now = std::time::SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .unwrap()
                                                        .as_secs();
                                                    let days_since_epoch = now / 86400;
                                                    let mut hasher = DefaultHasher::new();
                                                    days_since_epoch.hash(&mut hasher);
                                                    self.board.seed = Some(hasher.finish());
                                                }

                                                self.screen = AppScreen::Gameplay;
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(10.0);
                    }
                    ui.add_space(20.0);
                });
        });
    }

    fn show_mastery_challenges(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();
        let success = self.theme.colors.success();

        if ui_tokens::screen_header(
            ui,
            "MASTERY CHALLENGES",
            text_primary,
            border,
            card_bg,
            |_| {},
        ) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::MainMenu;
        }

        ui.add_space(14.0);

        let catalog = MasteryChallenge::catalog();
        let total_count = catalog.len();
        let completed_count = catalog
            .iter()
            .filter(|c| self.challenge_completions.contains_key(&c.id))
            .count();
        let progress_pct = if total_count > 0 {
            completed_count as f32 / total_count as f32
        } else {
            0.0
        };

        ui_tokens::centered_container(ui, 1020.0, |ui| {
            // Header stats & progress bar
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Completed {} / {} Challenges ({:.0}%)",
                        completed_count,
                        total_count,
                        progress_pct * 100.0
                    ))
                    .size(15.0)
                    .strong()
                    .color(text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_xp_earned: i64 = catalog
                        .iter()
                        .filter(|c| self.challenge_completions.contains_key(&c.id))
                        .map(|c| c.xp_reward)
                        .sum();
                    ui.label(
                        egui::RichText::new(format!("⭐ +{} XP Earned", total_xp_earned))
                            .size(14.0)
                            .strong()
                            .color(accent),
                    );
                });
            });
            ui.add_space(8.0);
            let pb = egui::ProgressBar::new(progress_pct).fill(accent);
            ui.add_sized([ui.available_width(), 10.0], pb);

            ui.add_space(14.0);

            // Tier Filter Pills
            ui.horizontal(|ui| {
                let filters = [
                    (None, format!("All ({})", total_count)),
                    (Some(ChallengeTier::Beginner), "Beginner".to_string()),
                    (
                        Some(ChallengeTier::Intermediate),
                        "Intermediate".to_string(),
                    ),
                    (Some(ChallengeTier::Advanced), "Advanced".to_string()),
                    (Some(ChallengeTier::Expert), "Expert".to_string()),
                    (Some(ChallengeTier::Master), "Master".to_string()),
                ];

                for (tier_opt, label) in filters {
                    let is_active = self.challenge_tier_filter == tier_opt;
                    let btn =
                        egui::Button::new(egui::RichText::new(label).size(13.0).strong().color(
                            if is_active {
                                egui::Color32::WHITE
                            } else {
                                text_primary
                            },
                        ))
                        .fill(if is_active { accent } else { card_bg })
                        .stroke(if is_active {
                            egui::Stroke::NONE
                        } else {
                            egui::Stroke::new(1.0, border)
                        })
                        .corner_radius(12.0)
                        .min_size(egui::vec2(95.0, 30.0));

                    if ui.add(btn).clicked() {
                        self.audio.play(SoundEffect::UiClick);
                        self.challenge_tier_filter = tier_opt;
                    }
                    ui.add_space(6.0);
                }
            });

            ui.add_space(14.0);

            // Responsive 2-column Grid of Challenge Cards
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let col_spacing = 16.0;
                    let card_frame_w = ((ui.available_width() - col_spacing) / 2.0)
                        .floor()
                        .max(240.0);
                    let card_inner_w = (card_frame_w - 32.0).max(200.0);

                    let filtered_challenges: Vec<_> = catalog
                        .into_iter()
                        .filter(|c| {
                            if let Some(tier) = self.challenge_tier_filter {
                                c.tier == tier
                            } else {
                                true
                            }
                        })
                        .collect();

                    for chunk in filtered_challenges.chunks(2) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = col_spacing;
                            for c in chunk {
                                let is_completed = self.challenge_completions.contains_key(&c.id);
                                let completion_record = self.challenge_completions.get(&c.id);

                                let card_stroke = if is_completed {
                                    egui::Stroke::new(1.5, success)
                                } else {
                                    egui::Stroke::new(1.0, border)
                                };

                                egui::Frame::new()
                                    .fill(card_bg)
                                    .stroke(card_stroke)
                                    .corner_radius(12.0)
                                    .inner_margin(16.0)
                                    .show(ui, |ui| {
                                        ui.set_width(card_inner_w);
                                        ui.vertical(|ui| {
                                            // Top row: Tier Badge + Category Pill + XP Badge
                                            ui.horizontal(|ui| {
                                                let (tier_bg, tier_name) = match c.tier {
                                                    ChallengeTier::Beginner => (
                                                        egui::Color32::from_rgb(80, 200, 120),
                                                        "BEGINNER",
                                                    ),
                                                    ChallengeTier::Intermediate => (
                                                        egui::Color32::from_rgb(74, 144, 226),
                                                        "INTERMEDIATE",
                                                    ),
                                                    ChallengeTier::Advanced => (
                                                        egui::Color32::from_rgb(155, 89, 182),
                                                        "ADVANCED",
                                                    ),
                                                    ChallengeTier::Expert => (
                                                        egui::Color32::from_rgb(230, 126, 34),
                                                        "EXPERT",
                                                    ),
                                                    ChallengeTier::Master => (
                                                        egui::Color32::from_rgb(231, 76, 60),
                                                        "MASTER",
                                                    ),
                                                };

                                                egui::Frame::new()
                                                    .fill(tier_bg)
                                                    .corner_radius(6.0)
                                                    .inner_margin(egui::vec2(6.0, 2.0))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            egui::RichText::new(tier_name)
                                                                .size(10.0)
                                                                .strong()
                                                                .color(egui::Color32::WHITE),
                                                        );
                                                    });

                                                ui.add_space(6.0);
                                                ui.label(
                                                    egui::RichText::new(c.category.name())
                                                        .size(12.0)
                                                        .color(text_muted),
                                                );

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "+{} XP",
                                                                c.xp_reward
                                                            ))
                                                            .size(13.0)
                                                            .strong()
                                                            .color(accent),
                                                        );
                                                    },
                                                );
                                            });

                                            ui.add_space(8.0);

                                            // Title
                                            ui.label(
                                                egui::RichText::new(&c.title)
                                                    .size(18.0)
                                                    .strong()
                                                    .color(text_primary),
                                            );

                                            ui.add_space(4.0);

                                            // Description
                                            ui.label(
                                                egui::RichText::new(&c.description)
                                                    .size(13.0)
                                                    .color(text_muted),
                                            );

                                            ui.add_space(10.0);

                                            // Rules / Modifiers row
                                            ui.horizontal_wrapped(|ui| {
                                                let topo_str = match c.ruleset.topology {
                                                    minecore::board::Geometry::Square => {
                                                        format!(
                                                            "Square {}×{}",
                                                            c.ruleset.width, c.ruleset.height
                                                        )
                                                    }
                                                    minecore::board::Geometry::Hex => {
                                                        format!("Hexagon R{}", c.ruleset.radius)
                                                    }
                                                };
                                                ui.label(
                                                    egui::RichText::new(format!("📐 {}", topo_str))
                                                        .size(11.0)
                                                        .color(text_muted),
                                                );
                                                ui.add_space(8.0);
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "💣 {} Mines",
                                                        c.ruleset.mine_count
                                                    ))
                                                    .size(11.0)
                                                    .color(text_muted),
                                                );

                                                match c.ruleset.flag_policy {
                                                    FlagPolicy::NoFlags => {
                                                        ui.add_space(8.0);
                                                        ui.label(
                                                            egui::RichText::new("🚫 No Flags")
                                                                .size(11.0)
                                                                .strong()
                                                                .color(self.theme.colors.danger()),
                                                        );
                                                    }
                                                    FlagPolicy::LimitedFlags { max_flags } => {
                                                        ui.add_space(8.0);
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "🚩 {} Flags",
                                                                max_flags
                                                            ))
                                                            .size(11.0)
                                                            .strong()
                                                            .color(accent),
                                                        );
                                                    }
                                                    FlagPolicy::Standard => {}
                                                }

                                                if c.ruleset.clue_policy
                                                    == CluePolicy::ExtendedRing2
                                                {
                                                    ui.add_space(8.0);
                                                    ui.label(
                                                        egui::RichText::new("🔵 Ring-2 Clues")
                                                            .size(11.0)
                                                            .strong()
                                                            .color(egui::Color32::from_rgb(
                                                                160, 110, 255,
                                                            )),
                                                    );
                                                }

                                                if let Some(target) = c.target_time_sec {
                                                    ui.add_space(8.0);
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "⏱ < {:.0}s",
                                                            target
                                                        ))
                                                        .size(11.0)
                                                        .color(accent),
                                                    );
                                                }
                                            });

                                            ui.add_space(12.0);

                                            // Bottom action row: Status badge + Start Button
                                            ui.horizontal(|ui| {
                                                if is_completed {
                                                    let best_text =
                                                        if let Some(rec) = completion_record {
                                                            format!(
                                                                "✓ COMPLETED ({:.1}s)",
                                                                rec.best_time_ms as f64 / 1000.0
                                                            )
                                                        } else {
                                                            "✓ COMPLETED".to_string()
                                                        };
                                                    egui::Frame::new()
                                                        .fill(success.linear_multiply(0.2))
                                                        .corner_radius(6.0)
                                                        .stroke(egui::Stroke::new(1.0, success))
                                                        .inner_margin(egui::vec2(8.0, 4.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new(best_text)
                                                                    .size(11.0)
                                                                    .strong()
                                                                    .color(success),
                                                            );
                                                        });
                                                } else {
                                                    egui::Frame::new()
                                                        .fill(card_bg)
                                                        .corner_radius(6.0)
                                                        .stroke(egui::Stroke::new(1.0, border))
                                                        .inner_margin(egui::vec2(8.0, 4.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new("UNSOLVED")
                                                                    .size(11.0)
                                                                    .strong()
                                                                    .color(text_muted),
                                                            );
                                                        });
                                                }

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        let play_btn = egui::Button::new(
                                                            egui::RichText::new("START CHALLENGE")
                                                                .size(13.0)
                                                                .strong()
                                                                .color(egui::Color32::WHITE),
                                                        )
                                                        .fill(accent)
                                                        .corner_radius(6.0)
                                                        .min_size(egui::vec2(140.0, 32.0));

                                                        if ui.add(play_btn).clicked() {
                                                            self.audio.play(SoundEffect::UiClick);
                                                            if let Some(ref prof) =
                                                                self.active_profile
                                                            {
                                                                if let Some(ref db) = self.db {
                                                                    let _ = db
                                                                        .record_challenge_attempt(
                                                                            prof.id, &c.id,
                                                                        );
                                                                }
                                                            }
                                                            self.active_challenge_id =
                                                                Some(c.id.clone());
                                                            self.active_hint = None;
                                                            self.blitz_combo = 0;
                                                            self.board = Board::from_ruleset(
                                                                c.ruleset.clone(),
                                                            );
                                                            self.mode_state = mode::ModeState::new(
                                                                mode::GameModeId::Classic,
                                                            );
                                                            if let Some(t) = c.target_time_sec {
                                                                self.mode_state.time_limit_sec =
                                                                    Some(t);
                                                            } else {
                                                                self.mode_state.time_limit_sec =
                                                                    None;
                                                            }
                                                            self.mode_state.lives =
                                                                c.ruleset.lives as i32;
                                                            self.screen = AppScreen::Gameplay;
                                                        }
                                                    },
                                                );
                                            });
                                        });
                                    });
                            }
                        });
                        ui.add_space(10.0);
                    }
                    ui.add_space(30.0);
                });
        });
    }

    fn show_sandbox_lab(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        if ui_tokens::screen_header(
            ui,
            "CUSTOM SANDBOX LABORATORY",
            text_primary,
            border,
            card_bg,
            |_| {},
        ) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::GameModes;
        }

        ui.add_space(10.0);

        ui_tokens::centered_container(ui, 860.0, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui_tokens::card_frame(card_bg, border, 12.0, 14.0).show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("🔬 Experimental Board Configuration")
                                .size(17.0)
                                .strong()
                                .color(text_primary),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new("Tinker with dimensions, topologies, opening mechanics, flagging constraints, and extended clues.")
                                .size(12.0)
                                .color(text_muted),
                        );

                        ui.add_space(12.0);

                        // 1. Geometry Selection
                        ui.label(egui::RichText::new("1. Board Topology").size(14.0).strong().color(text_primary));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let is_sq = self.sandbox_geometry == minecore::board::Geometry::Square;
                            let sq_btn = egui::Button::new(egui::RichText::new("Square Grid (Cartesian)").strong().color(if is_sq { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_sq { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(8.0)
                                .min_size(egui::vec2(190.0, 32.0));
                            if ui.add(sq_btn).clicked() {
                                self.sandbox_geometry = minecore::board::Geometry::Square;
                            }

                            ui.add_space(10.0);

                            let is_hex = self.sandbox_geometry == minecore::board::Geometry::Hex;
                            let hex_btn = egui::Button::new(egui::RichText::new("Hexagonal Grid (Axial)").strong().color(if is_hex { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_hex { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(8.0)
                                .min_size(egui::vec2(190.0, 32.0));
                            if ui.add(hex_btn).clicked() {
                                self.sandbox_geometry = minecore::board::Geometry::Hex;
                            }
                        });

                        ui.add_space(12.0);

                        // 2. Dimensions & Mines
                        ui.label(egui::RichText::new("2. Dimensions & Mine Density").size(14.0).strong().color(text_primary));
                        ui.add_space(4.0);
                        let total_cells = match self.sandbox_geometry {
                            minecore::board::Geometry::Square => {
                                ui.horizontal(|ui| {
                                    ui.label("Width:");
                                    ui.add(egui::Slider::new(&mut self.sandbox_width, 5..=40).text("cols"));
                                    ui.add_space(16.0);
                                    ui.label("Height:");
                                    ui.add(egui::Slider::new(&mut self.sandbox_height, 5..=30).text("rows"));
                                });
                                self.sandbox_width * self.sandbox_height
                            }
                            minecore::board::Geometry::Hex => {
                                ui.horizontal(|ui| {
                                    ui.label("Hex Radius:");
                                    ui.add(egui::Slider::new(&mut self.sandbox_radius, 3..=12).text("rings"));
                                });
                                3 * self.sandbox_radius * (self.sandbox_radius + 1) + 1
                            }
                        };

                        let max_mines = (total_cells.saturating_sub(10)).max(1);
                        self.sandbox_mines = self.sandbox_mines.clamp(1, max_mines);

                        ui.add_space(4.0);
                        let density = (self.sandbox_mines as f32 / total_cells as f32) * 100.0;
                        ui.horizontal(|ui| {
                            ui.label("Mines:");
                            ui.add(egui::Slider::new(&mut self.sandbox_mines, 1..=max_mines).text(format!("mines ({:.1}% density)", density)));
                        });

                        ui.add_space(12.0);

                        // 3. Opening Policy
                        ui.label(egui::RichText::new("3. First-Click Opening Policy").size(14.0).strong().color(text_primary));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let policies = [
                                (FirstClickPolicy::StandardSafe, "Standard Safe", "Initial cell never contains a mine."),
                                (FirstClickPolicy::GuaranteedZeroOpening, "Guaranteed 0 Opening", "Forces an opening with 0 adjacent mines."),
                                (FirstClickPolicy::ValidatedDeductionOpening, "Validated Logic Opening", "Guarantees a logical starting cluster."),
                            ];
                            for (pol, label, _tooltip) in policies {
                                let is_active = self.sandbox_opening_policy == pol;
                                let btn = egui::Button::new(egui::RichText::new(label).strong().color(if is_active { egui::Color32::WHITE } else { text_primary }))
                                    .fill(if is_active { accent } else { card_bg })
                                    .stroke(egui::Stroke::new(1.0, border))
                                    .corner_radius(6.0)
                                    .min_size(egui::vec2(150.0, 30.0));
                                if ui.add(btn).clicked() {
                                    self.sandbox_opening_policy = pol;
                                }
                                ui.add_space(8.0);
                            }
                        });

                        ui.add_space(12.0);

                        // 4. Flagging Policy
                        ui.label(egui::RichText::new("4. Flagging Mechanics").size(14.0).strong().color(text_primary));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let is_std = matches!(self.sandbox_flag_policy, FlagPolicy::Standard);
                            let is_none = matches!(self.sandbox_flag_policy, FlagPolicy::NoFlags);
                            let is_ltd = matches!(self.sandbox_flag_policy, FlagPolicy::LimitedFlags { .. });

                            let std_btn = egui::Button::new(egui::RichText::new("Standard (Unlimited)").strong().color(if is_std { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_std { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(140.0, 30.0));
                            if ui.add(std_btn).clicked() {
                                self.sandbox_flag_policy = FlagPolicy::Standard;
                            }
                            ui.add_space(8.0);

                            let no_btn = egui::Button::new(egui::RichText::new("🚫 Precision (No Flags)").strong().color(if is_none { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_none { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(150.0, 30.0));
                            if ui.add(no_btn).clicked() {
                                self.sandbox_flag_policy = FlagPolicy::NoFlags;
                            }
                            ui.add_space(8.0);

                            let ltd_btn = egui::Button::new(egui::RichText::new("🚩 Limited Flags").strong().color(if is_ltd { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_ltd { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(130.0, 30.0));
                            if ui.add(ltd_btn).clicked() {
                                self.sandbox_flag_policy = FlagPolicy::LimitedFlags { max_flags: (self.sandbox_mines / 2).max(1) };
                            }
                        });

                        if let FlagPolicy::LimitedFlags { ref mut max_flags } = self.sandbox_flag_policy {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label("Maximum Allowed Flags:");
                                ui.add(egui::Slider::new(max_flags, 1..=self.sandbox_mines));
                            });
                        }

                        ui.add_space(12.0);

                        // 5. Clue & Deduction Policies
                        ui.label(egui::RichText::new("5. Clue Cell Mechanics").size(14.0).strong().color(text_primary));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let is_std_clue = self.sandbox_clue_policy == CluePolicy::StandardOnly;
                            let std_c_btn = egui::Button::new(egui::RichText::new("Standard Adjacent (Distance 1)").strong().color(if is_std_clue { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_std_clue { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(180.0, 30.0));
                            if ui.add(std_c_btn).clicked() {
                                self.sandbox_clue_policy = CluePolicy::StandardOnly;
                            }
                            ui.add_space(8.0);

                            let is_r2_clue = self.sandbox_clue_policy == CluePolicy::ExtendedRing2;
                            let r2_c_btn = egui::Button::new(egui::RichText::new("🔵 Extended Ring-2 Clues (Distance 2)").strong().color(if is_r2_clue { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_r2_clue { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(210.0, 30.0));
                            if ui.add(r2_c_btn).clicked() {
                                self.sandbox_clue_policy = CluePolicy::ExtendedRing2;
                            }
                        });

                        ui.add_space(12.0);

                        // 6. Logic Guarantee
                        ui.label(egui::RichText::new("6. Logic Guarantee").size(14.0).strong().color(text_primary));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let is_none_log = self.sandbox_logic_guarantee == LogicGuarantee::StandardProbabilistic;
                            let none_btn = egui::Button::new(egui::RichText::new("Standard Random").strong().color(if is_none_log { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_none_log { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(140.0, 30.0));
                            if ui.add(none_btn).clicked() {
                                self.sandbox_logic_guarantee = LogicGuarantee::StandardProbabilistic;
                            }
                            ui.add_space(8.0);

                            let is_cert = self.sandbox_logic_guarantee == LogicGuarantee::CertifiedNoGuess;
                            let cert_btn = egui::Button::new(egui::RichText::new("✓ Certified Solvable (No Guess)").strong().color(if is_cert { egui::Color32::WHITE } else { text_primary }))
                                .fill(if is_cert { accent } else { card_bg })
                                .stroke(egui::Stroke::new(1.0, border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(180.0, 30.0));
                            if ui.add(cert_btn).clicked() {
                                self.sandbox_logic_guarantee = LogicGuarantee::CertifiedNoGuess;
                            }
                        });

                        ui.add_space(16.0);

                        // Launch Button
                        let launch_btn = egui::Button::new(
                            egui::RichText::new("🚀 LAUNCH EXPERIMENT")
                                .size(17.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(accent)
                        .corner_radius(8.0)
                        .min_size(egui::vec2(ui.available_width(), 48.0));

                        if ui.add(launch_btn).clicked() {
                            self.audio.play(SoundEffect::UiClick);
                            let ruleset = Ruleset {
                                topology: self.sandbox_geometry,
                                width: self.sandbox_width,
                                height: self.sandbox_height,
                                radius: self.sandbox_radius,
                                mine_count: self.sandbox_mines,
                                first_click_policy: self.sandbox_opening_policy,
                                flag_policy: self.sandbox_flag_policy,
                                clue_policy: self.sandbox_clue_policy,
                                logic_guarantee: self.sandbox_logic_guarantee,
                                assist_policy: AssistPolicy::Allowed,
                                time_limit_sec: None,
                                lives: 1,
                            };
                            self.active_challenge_id = None;
                            self.active_hint = None;
                            self.blitz_combo = 0;
                            self.board = Board::from_ruleset(ruleset);
                            self.mode_state = mode::ModeState::new(mode::GameModeId::Sandbox);
                            self.screen = AppScreen::Gameplay;
                        }
                    });
                    ui.add_space(30.0);
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

        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.25);
            egui::Frame::new()
                .fill(card_bg)
                .stroke(egui::Stroke::new(1.0, border))
                .corner_radius(16.0)
                .inner_margin(32.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Preparing Solvable Board...")
                                .size(24.0)
                                .strong()
                                .color(text_primary),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "Generating a guaranteed deduction path without guessing",
                            )
                            .size(15.0)
                            .color(text_muted),
                        );
                        ui.add_space(24.0);
                        ui.spinner();
                        ui.add_space(28.0);
                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("Cancel").size(16.0).color(text_primary),
                        )
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(8.0)
                        .min_size(egui::vec2(120.0, 38.0));
                        if ui.add(cancel_btn).clicked() {
                            self.audio.play(SoundEffect::UiClick);
                            self.generation_rx = None;
                            self.screen = AppScreen::MainMenu;
                        }
                    });
                });
        });
    }
    fn show_theme_studio(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        if ui_tokens::screen_header(
            ui,
            "CUSTOMIZATION HUB",
            text_primary,
            border,
            card_bg,
            |ui| {
                if ui
                    .button(
                        egui::RichText::new("✨ Profile Cosmetics")
                            .size(15.0)
                            .strong()
                            .color(text_primary),
                    )
                    .clicked()
                {
                    self.audio.play(SoundEffect::UiClick);
                    self.screen = AppScreen::ProfileCosmetics;
                }
            },
        ) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::MainMenu;
            if let Some(ref prof) = self.active_profile {
                if let Some(ref db) = self.db {
                    let _ = db.set_profile_theme(prof.id, &self.theme.id);
                }
            }
        }

        ui.add_space(8.0);

        // 3-Column Studio Layout
        ui.horizontal_top(|ui| {
            // Column 1: Theme Presets & Custom Library (~250px)
            ui.vertical(|ui| {
                ui.set_width(250.0);
                ui_tokens::card_frame(card_bg, border, 12.0, 16.0).show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Theme Presets")
                            .size(18.0)
                            .strong()
                            .color(text_primary),
                    );
                    ui.add_space(8.0);

                    let presets = vec![
                        crate::theme::Theme::soft_circuit(),
                        crate::theme::Theme::obsidian(),
                        crate::theme::Theme::aurora(),
                        crate::theme::Theme::paper(),
                        crate::theme::Theme::retro(),
                        crate::theme::Theme::neon_arcade(),
                    ];

                    for preset in presets {
                        let is_active = self.theme.id == preset.id;
                        let btn = egui::Button::new(
                            egui::RichText::new(format!(
                                "{}  {}",
                                if is_active { "●" } else { "○" },
                                &preset.name
                            ))
                            .size(15.0)
                            .color(if is_active {
                                egui::Color32::WHITE
                            } else {
                                text_primary
                            }),
                        )
                        .fill(if is_active { accent } else { card_bg })
                        .stroke(if is_active {
                            egui::Stroke::NONE
                        } else {
                            egui::Stroke::new(1.0, border)
                        })
                        .corner_radius(6.0)
                        .min_size(egui::vec2(216.0, 36.0));

                        if ui.add(btn).clicked() {
                            self.audio.play(SoundEffect::UiClick);
                            self.set_active_theme(&preset.id);
                        }
                        ui.add_space(4.0);
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Custom Themes")
                            .size(18.0)
                            .strong()
                            .color(text_primary),
                    );
                    ui.add_space(8.0);

                    if let Some(ref prof) = self.active_profile {
                        if let Some(ref db) = self.db {
                            if let Ok(customs) = db.get_custom_themes(prof.id) {
                                for (id, name, data) in customs {
                                    let is_active = self.theme.id == id;
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{}  {}",
                                            if is_active { "●" } else { "○" },
                                            &name
                                        ))
                                        .size(15.0)
                                        .color(
                                            if is_active {
                                                egui::Color32::WHITE
                                            } else {
                                                text_primary
                                            },
                                        ),
                                    )
                                    .fill(if is_active { accent } else { card_bg })
                                    .corner_radius(6.0)
                                    .min_size(egui::vec2(208.0, 34.0));

                                    if ui.add(btn).clicked() {
                                        if let Ok(mut imported) =
                                            serde_json::from_str::<crate::theme::Theme>(&data)
                                        {
                                            imported.is_custom = true;
                                            self.theme = imported;
                                        }
                                    }
                                    ui.add_space(4.0);
                                }
                            }
                        }
                    }

                    ui.add_space(10.0);
                    if ui
                        .add_sized([208.0, 32.0], egui::Button::new("+ Create New Custom"))
                        .clicked()
                    {
                        let mut new_theme = self.theme.clone();
                        new_theme.is_custom = true;
                        new_theme.id = format!(
                            "custom_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                        new_theme.name = "Custom Theme".into();
                        self.theme = new_theme;
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([100.0, 30.0], egui::Button::new("Export"))
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                let data = serde_json::to_string_pretty(&self.theme).unwrap();
                                let _ = std::fs::write(path, data);
                            }
                        }
                        if ui
                            .add_sized([100.0, 30.0], egui::Button::new("Import"))
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                if let Ok(data) = std::fs::read_to_string(path) {
                                    if let Ok(mut imported) =
                                        serde_json::from_str::<crate::theme::Theme>(&data)
                                    {
                                        imported.is_custom = true;
                                        imported.id = format!(
                                            "custom_{}",
                                            std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs()
                                        );
                                        self.theme = imported;
                                    }
                                }
                            }
                        }
                    });

                    if self.theme.is_custom {
                        ui.add_space(6.0);
                        if ui
                            .add_sized([208.0, 32.0], egui::Button::new("💾 Save to Profile"))
                            .clicked()
                        {
                            if let Some(ref prof) = self.active_profile {
                                if let Some(ref db) = self.db {
                                    let data = serde_json::to_string(&self.theme).unwrap();
                                    let _ = db.save_custom_theme(
                                        prof.id,
                                        &self.theme.id,
                                        &self.theme.name,
                                        &data,
                                    );
                                }
                            }
                        }
                    }
                });
            });

            ui.add_space(16.0);

            // Column 2: Theme Customizer (~380px)
            ui.vertical(|ui| {
                ui.set_width(380.0);
                ui_tokens::card_frame(card_bg, border, 12.0, 18.0).show(ui, |ui| {
                    if self.theme.is_custom {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Theme Name:")
                                    .strong()
                                    .color(text_primary),
                            );
                            ui.text_edit_singleline(&mut self.theme.name);
                        });
                        ui.add_space(10.0);
                    }

                    // Tab bar: Colors | Geometry | Numbers
                    ui.horizontal(|ui| {
                        let tabs = [
                            (ThemeStudioTab::Colors, "🎨 Colors"),
                            (ThemeStudioTab::Geometry, "📐 Geometry"),
                            (ThemeStudioTab::Numbers, "🔢 Numbers"),
                        ];
                        for (tab, label) in tabs {
                            let is_active = self.theme_studio_tab == tab;
                            let btn = egui::Button::new(
                                egui::RichText::new(label).size(15.0).strong().color(
                                    if is_active {
                                        egui::Color32::WHITE
                                    } else {
                                        text_primary
                                    },
                                ),
                            )
                            .fill(if is_active { accent } else { card_bg })
                            .corner_radius(6.0)
                            .min_size(egui::vec2(106.0, 34.0));

                            if ui.add(btn).clicked() {
                                self.theme_studio_tab = tab;
                            }
                        }
                    });

                    ui.add_space(14.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(480.0)
                        .show(ui, |ui| match self.theme_studio_tab {
                            ThemeStudioTab::Colors => {
                                let color_row =
                                    |ui: &mut egui::Ui, name: &str, rgb: &mut (u8, u8, u8)| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(name)
                                                    .size(15.0)
                                                    .color(text_primary),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    color_picker(ui, rgb);
                                                },
                                            );
                                        });
                                        ui.add_space(6.0);
                                    };

                                color_row(ui, "Background", &mut self.theme.colors.bg);
                                color_row(ui, "Menu Background", &mut self.theme.colors.menu_bg);
                                color_row(ui, "Card Background", &mut self.theme.colors.card_bg);
                                color_row(ui, "Border Color", &mut self.theme.colors.border);
                                color_row(ui, "Covered Cell", &mut self.theme.colors.covered);
                                color_row(ui, "Revealed Cell", &mut self.theme.colors.revealed);
                                color_row(ui, "Text Primary", &mut self.theme.colors.text_primary);
                                color_row(ui, "Text Muted", &mut self.theme.colors.text_muted);
                                color_row(ui, "Accent Color", &mut self.theme.colors.accent);
                                color_row(ui, "Danger Color", &mut self.theme.colors.danger);
                                color_row(ui, "Success Color", &mut self.theme.colors.success);
                                color_row(ui, "Button Fill", &mut self.theme.colors.button_bg);
                            }
                            ThemeStudioTab::Geometry => {
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.theme.geometry.cell_spacing,
                                        0.0..=10.0,
                                    )
                                    .text("Cell Spacing"),
                                );
                                ui.add_space(10.0);
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.theme.geometry.square_radius,
                                        0.0..=20.0,
                                    )
                                    .text("Square Radius"),
                                );
                                ui.add_space(10.0);
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.theme.geometry.shadow_depth,
                                        0.0..=10.0,
                                    )
                                    .text("Shadow Depth"),
                                );
                                ui.add_space(10.0);
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.theme.geometry.animation_speed,
                                        0.5..=3.0,
                                    )
                                    .text("Animation Speed"),
                                );
                                ui.add_space(10.0);
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.theme.geometry.border_thickness,
                                        0.0..=5.0,
                                    )
                                    .text("Border Thickness"),
                                );
                            }
                            ThemeStudioTab::Numbers => {
                                for i in 0..8 {
                                    if i < self.theme.colors.number_colors.len() {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(format!("Number {}", i + 1))
                                                    .size(15.0)
                                                    .color(text_primary),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    color_picker(
                                                        ui,
                                                        &mut self.theme.colors.number_colors[i],
                                                    );
                                                },
                                            );
                                        });
                                        ui.add_space(6.0);
                                    }
                                }
                            }
                        });
                });
            });

            ui.add_space(16.0);

            // Column 3: Live Interactive Board Preview (~380px)
            ui.vertical(|ui| {
                let avail_w = ui.available_width().max(340.0);
                ui_tokens::card_frame(card_bg, border, 12.0, 18.0).show(ui, |ui| {
                    ui.set_width(avail_w - 30.0);
                    ui.label(
                        egui::RichText::new("Live Board Preview")
                            .size(18.0)
                            .strong()
                            .color(text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Real-time preview of active theme geometry & palette")
                            .size(13.0)
                            .color(text_muted),
                    );
                    ui.add_space(14.0);

                    // 1. Square Preview Grid (5x3 realistic board)
                    ui.label(
                        egui::RichText::new("Square Grid (5×3):")
                            .size(15.0)
                            .strong()
                            .color(text_primary),
                    );
                    ui.add_space(8.0);

                    let cell_sz = 36.0;
                    let spacing = self.theme.geometry.cell_spacing.clamp(2.0, 8.0);
                    let grid_w = 5.0 * (cell_sz + spacing);
                    let grid_h = 3.0 * (cell_sz + spacing);
                    let (sq_rect, _) =
                        ui.allocate_exact_size(egui::vec2(grid_w, grid_h), egui::Sense::hover());

                    // (x, y, state: 0=covered, 1=flagged, 2=revealed blank, 3=rev 1, 4=rev 2, 5=rev 3)
                    let square_sample = [
                        (0, 0, 0),
                        (1, 0, 0),
                        (2, 0, 1),
                        (3, 0, 0),
                        (4, 0, 0),
                        (0, 1, 3),
                        (1, 1, 4),
                        (2, 1, 3),
                        (3, 1, 0),
                        (4, 1, 0),
                        (0, 2, 2),
                        (1, 2, 3),
                        (2, 2, 0),
                        (3, 2, 1),
                        (4, 2, 0),
                    ];

                    for (gx, gy, kind) in square_sample {
                        let center = sq_rect.min
                            + egui::vec2(
                                gx as f32 * (cell_sz + spacing) + cell_sz / 2.0,
                                gy as f32 * (cell_sz + spacing) + cell_sz / 2.0,
                            );
                        let cell_rect =
                            egui::Rect::from_center_size(center, egui::vec2(cell_sz, cell_sz));

                        // Shadow
                        if self.theme.geometry.shadow_depth > 0.0 {
                            let shadow = cell_rect
                                .translate(egui::vec2(0.0, self.theme.geometry.shadow_depth * 0.5));
                            ui.painter().rect_filled(
                                shadow,
                                self.theme.geometry.square_radius,
                                egui::Color32::from_black_alpha(20),
                            );
                        }

                        let is_revealed = kind >= 2;
                        let fill = if is_revealed {
                            self.theme.colors.revealed()
                        } else {
                            self.theme.colors.covered()
                        };
                        ui.painter().rect_filled(
                            cell_rect,
                            self.theme.geometry.square_radius,
                            fill,
                        );

                        if self.theme.geometry.border_thickness > 0.0 {
                            ui.painter().rect_stroke(
                                cell_rect,
                                self.theme.geometry.square_radius,
                                egui::Stroke::new(
                                    self.theme.geometry.border_thickness,
                                    self.theme.colors.border(),
                                ),
                                egui::StrokeKind::Inside,
                            );
                        }

                        if kind == 1 {
                            // Flagged
                            let base_y = center.y + cell_sz * 0.22;
                            let tip_y = center.y - cell_sz * 0.25;
                            let flag_right = center.x + cell_sz * 0.22;
                            ui.painter().line_segment(
                                [egui::pos2(center.x, base_y), egui::pos2(center.x, tip_y)],
                                egui::Stroke::new(2.0, self.theme.colors.text_primary()),
                            );
                            ui.painter().add(egui::Shape::convex_polygon(
                                vec![
                                    egui::pos2(center.x, tip_y),
                                    egui::pos2(flag_right, tip_y + cell_sz * 0.12),
                                    egui::pos2(center.x, tip_y + cell_sz * 0.24),
                                ],
                                self.theme.colors.danger(),
                                egui::Stroke::NONE,
                            ));
                        } else if kind == 3 {
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "1",
                                egui::FontId::proportional(cell_sz * 0.65),
                                self.theme.colors.number_color(1),
                            );
                        } else if kind == 4 {
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "2",
                                egui::FontId::proportional(cell_sz * 0.65),
                                self.theme.colors.number_color(2),
                            );
                        } else if kind == 5 {
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "3",
                                egui::FontId::proportional(cell_sz * 0.65),
                                self.theme.colors.number_color(3),
                            );
                        }
                    }

                    ui.add_space(20.0);

                    // 2. Hexagonal Preview Cluster (7-cell rosette)
                    ui.label(
                        egui::RichText::new("Hexagonal Cluster:")
                            .size(15.0)
                            .strong()
                            .color(text_primary),
                    );
                    ui.add_space(8.0);

                    let hex_r = 20.0;
                    let (hex_canvas, _) = ui.allocate_exact_size(
                        egui::vec2(avail_w - 40.0, 110.0),
                        egui::Sense::hover(),
                    );
                    let hex_origin = hex_canvas.min + egui::vec2(130.0, 55.0);

                    // Axial coordinates (q, r, kind: 0=covered, 1=flagged, 2=blank, 3=rev1, 4=rev2, 5=rev3)
                    let hex_sample = [
                        (0, 0, 4),  // Center: "2"
                        (1, 0, 0),  // Right: covered
                        (1, -1, 3), // Top-Right: "1"
                        (0, -1, 0), // Top-Left: covered
                        (-1, 0, 1), // Left: flagged
                        (-1, 1, 5), // Bottom-Left: "3"
                        (0, 1, 0),  // Bottom-Right: covered
                    ];

                    for (q, r, kind) in hex_sample {
                        let cx = hex_r * 3.0_f32.sqrt() * (q as f32 + r as f32 / 2.0);
                        let cy = hex_r * 1.5 * r as f32;
                        let center = hex_origin + egui::vec2(cx, cy);

                        let points = coords::hex_vertices(center, hex_r);
                        if self.theme.geometry.shadow_depth > 0.0 {
                            let shadow_pts: Vec<egui::Pos2> =
                                points.iter().map(|p| *p + egui::vec2(0.0, 2.0)).collect();
                            ui.painter().add(egui::Shape::convex_polygon(
                                shadow_pts,
                                egui::Color32::from_black_alpha(20),
                                egui::Stroke::NONE,
                            ));
                        }

                        let is_revealed = kind >= 2;
                        let fill = if is_revealed {
                            self.theme.colors.revealed()
                        } else {
                            self.theme.colors.covered()
                        };
                        ui.painter().add(egui::Shape::convex_polygon(
                            points.clone(),
                            fill,
                            egui::Stroke::NONE,
                        ));

                        if self.theme.geometry.border_thickness > 0.0 {
                            ui.painter().add(egui::Shape::closed_line(
                                points,
                                egui::Stroke::new(
                                    self.theme.geometry.border_thickness,
                                    self.theme.colors.border(),
                                ),
                            ));
                        }

                        if kind == 1 {
                            let base_y = center.y + hex_r * 0.3;
                            let tip_y = center.y - hex_r * 0.35;
                            let flag_right = center.x + hex_r * 0.3;
                            ui.painter().line_segment(
                                [egui::pos2(center.x, base_y), egui::pos2(center.x, tip_y)],
                                egui::Stroke::new(2.0, self.theme.colors.text_primary()),
                            );
                            ui.painter().add(egui::Shape::convex_polygon(
                                vec![
                                    egui::pos2(center.x, tip_y),
                                    egui::pos2(flag_right, tip_y + hex_r * 0.15),
                                    egui::pos2(center.x, tip_y + hex_r * 0.3),
                                ],
                                self.theme.colors.danger(),
                                egui::Stroke::NONE,
                            ));
                        } else if kind == 3 {
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "1",
                                egui::FontId::proportional(hex_r * 1.1),
                                self.theme.colors.number_color(1),
                            );
                        } else if kind == 4 {
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "2",
                                egui::FontId::proportional(hex_r * 1.1),
                                self.theme.colors.number_color(2),
                            );
                        } else if kind == 5 {
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "3",
                                egui::FontId::proportional(hex_r * 1.1),
                                self.theme.colors.number_color(3),
                            );
                        }
                    }
                });
            });
        });
    }
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        let time = ui.input(|i| i.time);
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();

        if ui_tokens::screen_header(ui, "SETTINGS", text_primary, border, card_bg, |ui| {
            egui::Frame::new()
                .fill(card_bg)
                .corner_radius(16.0)
                .stroke(egui::Stroke::new(1.0, border))
                .inner_margin(egui::vec2(16.0, 6.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&self.theme.name)
                            .size(14.0)
                            .strong()
                            .color(text_primary),
                    );
                });
        }) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::MainMenu;
        }

        ui.add_space(20.0);

        // Responsive Two-Column Layout
        ui.horizontal_top(|ui| {
            // Left Navigation Sidebar
            ui.vertical(|ui| {
                ui.set_width(190.0);

                let tabs = [
                    (SettingsTab::General, "General"),
                    (SettingsTab::Audio, "Audio"),
                    (SettingsTab::GameFeel, "Game Feel"),
                    (SettingsTab::Cursor, "Cursor"),
                    (SettingsTab::Appearance, "Appearance"),
                    (SettingsTab::Accessibility, "Accessibility"),
                ];

                for (tab, label) in tabs {
                    let is_active = self.settings_tab == tab;
                    let (bg, fg) = if is_active {
                        (accent, egui::Color32::WHITE)
                    } else {
                        (card_bg, text_primary)
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .size(16.0)
                            .strong()
                            .color(fg)
                    )
                    .fill(bg)
                    .corner_radius(10.0)
                    .stroke(if is_active { egui::Stroke::NONE } else { egui::Stroke::new(1.0, border) })
                    .min_size(egui::vec2(190.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.audio.play(SoundEffect::UiClick);
                        self.settings_tab = tab;
                    }
                    ui.add_space(8.0);
                }
            });

            ui.add_space(24.0);

            // Right Content Area
            ui.vertical(|ui| {
                let avail_width = ui.available_width();

                // Section Header & Subtitle
                let (title, subtitle) = match self.settings_tab {
                    SettingsTab::General => ("General", "Manage player profiles and default game rules"),
                    SettingsTab::Audio => ("Audio", "Adjust your sound experience"),
                    SettingsTab::GameFeel => ("Game Feel", "Tactile feedback, animations, and camera responsiveness"),
                    SettingsTab::Cursor => ("Cursor", "Hardware cursor behavior and kinetic motion trails"),
                    SettingsTab::Appearance => ("Appearance", "Color themes, visual geometry, and board presets"),
                    SettingsTab::Accessibility => ("Accessibility", "Motion comfort, contrast, and visual assists"),
                };

                ui.label(egui::RichText::new(title).size(26.0).strong().color(text_primary));
                ui.add_space(2.0);
                ui.label(egui::RichText::new(subtitle).size(15.0).color(text_muted));
                ui.add_space(16.0);

                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    match self.settings_tab {
                        SettingsTab::Audio => {
                            // Card 1: Volume Sliders
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);

                                // Master Volume
                                let master_pct = format!("{:.0}%", self.settings.master_volume * 100.0);
                                if ui_tokens::modern_slider_row(ui, "Master Volume", &mut self.settings.master_volume, 0.0..=1.0, &master_pct, accent, text_primary, text_muted) {
                                    self.sync_settings();
                                }
                                ui.add_space(14.0);

                                // Music Volume
                                let music_pct = format!("{:.0}%", self.settings.music_volume * 100.0);
                                if ui_tokens::modern_slider_row(ui, "Music", &mut self.settings.music_volume, 0.0..=1.0, &music_pct, accent, text_primary, text_muted) {
                                    self.sync_settings();
                                }
                                ui.add_space(14.0);

                                // Sound Effects Volume
                                let sfx_pct = format!("{:.0}%", self.settings.sfx_volume * 100.0);
                                if ui_tokens::modern_slider_row(ui, "Sound Effects", &mut self.settings.sfx_volume, 0.0..=1.0, &sfx_pct, accent, text_primary, text_muted) {
                                    self.sync_settings();
                                }
                                ui.add_space(14.0);

                                if ui.checkbox(&mut self.settings.mute_audio, "Mute All Audio").changed() {
                                    self.sync_settings();
                                }
                            });

                            ui.add_space(16.0);

                            // Card 2: Dynamic Sound
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("Dynamic Sound").size(18.0).strong().color(text_primary));
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("Sounds respond subtly to your gameplay").size(14.0).color(text_muted));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.checkbox(&mut self.settings.pitch_variation, "").changed() {
                                            self.sync_settings();
                                        }
                                    });
                                });

                                ui.add_space(16.0);
                                ui.separator();
                                ui.add_space(10.0);
                                ui.label(egui::RichText::new("Sound Synthesizer Test").size(15.0).strong().color(text_primary));
                                ui.add_space(8.0);
                                ui.horizontal_wrapped(|ui| {
                                    let btn_s = egui::vec2(100.0, 32.0);
                                    if ui.add_sized(btn_s, egui::Button::new("Click")).clicked() { self.audio.play(SoundEffect::TileClick); }
                                    if ui.add_sized(btn_s, egui::Button::new("Reveal 1")).clicked() { self.audio.play(SoundEffect::TileReveal(1)); }
                                    if ui.add_sized(btn_s, egui::Button::new("Reveal 4")).clicked() { self.audio.play(SoundEffect::TileReveal(4)); }
                                    if ui.add_sized(btn_s, egui::Button::new("Flag")).clicked() { self.audio.play(SoundEffect::FlagPlace); }
                                    if ui.add_sized(btn_s, egui::Button::new("Chord")).clicked() { self.audio.play(SoundEffect::Chord); }
                                    if ui.add_sized(btn_s, egui::Button::new("Explosion")).clicked() { self.audio.play(SoundEffect::Explosion); self.shake.add_trauma(0.8); }
                                    if ui.add_sized(btn_s, egui::Button::new("Victory")).clicked() { self.audio.play(SoundEffect::Victory); }
                                });
                            });
                        }
                        SettingsTab::General => {
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                ui.label(egui::RichText::new("Active Player Profile").size(18.0).strong().color(text_primary));
                                ui.add_space(8.0);
                                if let Some(ref prof) = self.active_profile {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&prof.avatar_id).size(42.0));
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(&prof.display_name).size(22.0).strong().color(text_primary));
                                            ui.label(egui::RichText::new(format!("Level {} • {} XP", prof.current_level, prof.total_xp)).size(14.0).color(accent));
                                        });
                                    });
                                } else {
                                    ui.label(egui::RichText::new("No active profile selected").color(text_muted));
                                }
                                ui.add_space(14.0);
                                if ui.button(egui::RichText::new("Manage Profiles & Cosmetics").size(15.0)).clicked() {
                                    self.audio.play(SoundEffect::UiClick);
                                    self.screen = AppScreen::Profiles;
                                }
                            });
                        }
                        SettingsTab::GameFeel => {
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                let shake_pct = format!("{:.0}%", self.settings.screen_shake * 100.0);
                                if ui_tokens::modern_slider_row(ui, "Screen Shake Intensity", &mut self.settings.screen_shake, 0.0..=2.0, &shake_pct, accent, text_primary, text_muted) {
                                    self.sync_settings();
                                }
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    if ui.button(egui::RichText::new("Test Screen Shake Impulse").size(14.0).strong()).clicked() {
                                        self.audio.play(SoundEffect::Explosion);
                                        self.shake.add_trauma(0.85);
                                    }
                                    ui.add_space(8.0);
                                    if self.settings.cursor.reduced_motion {
                                        ui.label(egui::RichText::new("⚠ Suppressed by Reduced Motion setting").size(13.0).color(self.theme.colors.danger()));
                                    } else if self.settings.screen_shake <= 0.01 {
                                        ui.label(egui::RichText::new("⚠ Intensity is 0% — increase slider to test impulse").size(13.0).color(text_muted));
                                    } else if self.shake.trauma > 0.02 {
                                        let trauma_pct = (self.shake.trauma * 100.0) as u32;
                                        ui.label(egui::RichText::new(format!("Impulse active (Trauma: {}%)", trauma_pct)).size(13.0).strong().color(accent));
                                    }
                                });

                                // Reactive live simulation target in Settings
                                if self.settings.screen_shake > 0.01 && !self.settings.cursor.reduced_motion {
                                    ui.add_space(10.0);
                                    let shake_offset = self.shake.get_offset(time);
                                    let is_shaking = self.shake.trauma > 0.02;
                                    let (target_rect, _) = ui.allocate_exact_size(egui::vec2(avail_width - 80.0, 36.0), egui::Sense::hover());
                                    let transformed_rect = target_rect.translate(shake_offset);
                                    ui.painter().rect(
                                        transformed_rect,
                                        8.0,
                                        if is_shaking { accent.linear_multiply(0.25) } else { self.theme.colors.button_bg() },
                                        egui::Stroke::new(1.5, if is_shaking { accent } else { border }),
                                        egui::epaint::StrokeKind::Inside,
                                    );
                                    ui.painter().text(
                                        transformed_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        if is_shaking { "⚡ SHAKING IMPULSE ACTIVE ⚡" } else { "Simulated Target (Reacts to screen shake impulse)" },
                                        egui::FontId::proportional(13.0),
                                        if is_shaking { accent } else { text_muted },
                                    );
                                }
                            });

                            ui.add_space(16.0);

                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("Particle Bursts & Debris").size(18.0).strong().color(text_primary));
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("Shockwaves, confetti, and debris particles during gameplay").size(14.0).color(text_muted));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.checkbox(&mut self.settings.particles_enabled, "").changed() {
                                            self.sync_settings();
                                        }
                                    });
                                });
                            });

                            ui.add_space(16.0);

                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                let speed_str = format!("{:.1}x", self.settings.animation_speed);
                                if ui_tokens::modern_slider_row(ui, "Reveal Animation Speed", &mut self.settings.animation_speed, 0.5..=3.0, &speed_str, accent, text_primary, text_muted) {
                                    self.sync_settings();
                                }
                            });
                        }
                        SettingsTab::Cursor => {
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("Theme-Aware Custom Cursor").size(18.0).strong().color(text_primary));
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("State-responsive custom hardware cursor").size(14.0).color(text_muted));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.checkbox(&mut self.settings.cursor.enabled, "").changed() {
                                            self.sync_settings();
                                        }
                                    });
                                });
                            });

                            ui.add_space(16.0);

                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                let size_str = format!("{:.1}x", self.settings.cursor.size);
                                if ui_tokens::modern_slider_row(ui, "Cursor Size", &mut self.settings.cursor.size, 0.5..=2.5, &size_str, accent, text_primary, text_muted) {
                                    self.sync_settings();
                                }
                                ui.add_space(14.0);
                                if ui.checkbox(&mut self.settings.cursor.trail, "Enable Dynamic Motion Trail").changed() {
                                    self.sync_settings();
                                }
                            });
                        }
                        SettingsTab::Appearance => {
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                ui.label(egui::RichText::new("Theme Presets").size(18.0).strong().color(text_primary));
                                ui.add_space(12.0);

                                let presets = [
                                    ("soft_circuit", "Soft Circuit"),
                                    ("obsidian", "Obsidian"),
                                    ("aurora", "Aurora"),
                                    ("paper", "Paper"),
                                    ("retro", "Retro"),
                                    ("neon_arcade", "Neon Arcade"),
                                ];

                                for (id, name) in presets {
                                    let is_active = self.theme.id == id;
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!("{}  {}", if is_active { "●" } else { "○" }, name))
                                            .size(16.0)
                                            .strong()
                                            .color(if is_active { egui::Color32::WHITE } else { text_primary })
                                    )
                                    .fill(if is_active { accent } else { card_bg })
                                    .corner_radius(8.0)
                                    .min_size(egui::vec2(avail_width - 50.0, 38.0));

                                    if ui.add(btn).clicked() {
                                        self.audio.play(SoundEffect::UiClick);
                                        self.set_active_theme(id);
                                    }
                                    ui.add_space(6.0);
                                }
                                ui.add_space(14.0);
                                if ui.button(egui::RichText::new("Open Full Customization Hub (Theme Studio)").size(15.0)).clicked() {
                                    self.audio.play(SoundEffect::UiClick);
                                    self.screen = AppScreen::ThemeStudio;
                                }
                            });
                        }
                        SettingsTab::Accessibility => {
                            ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                                ui.set_width(avail_width - 40.0);
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("Reduced Motion").size(18.0).strong().color(text_primary));
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("Disables screen shake and limits particle bursts").size(14.0).color(text_muted));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.checkbox(&mut self.settings.cursor.reduced_motion, "").changed() {
                                            self.sync_settings();
                                        }
                                    });
                                });
                                ui.add_space(16.0);
                                ui.separator();
                                ui.add_space(16.0);
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("High Contrast Mode").size(18.0).strong().color(text_primary));
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("Enhances board cell borders and number legibility").size(14.0).color(text_muted));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.checkbox(&mut self.settings.high_contrast, "").changed() {
                                            self.sync_settings();
                                        }
                                    });
                                });
                            });
                        }
                    }

                    ui.add_space(20.0);

                    // Bottom Action Buttons matching mockup: Save settings
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save settings")
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::WHITE)
                        )
                        .fill(accent)
                        .corner_radius(8.0)
                        .min_size(egui::vec2(140.0, 42.0));

                        if ui.add(save_btn).clicked() {
                            self.sync_settings();
                            self.save_current_settings();
                            self.settings_saved_feedback = Some(time);
                            self.audio.play(SoundEffect::UiClick);
                        }

                        ui.add_space(12.0);

                        let reset_btn = egui::Button::new(
                            egui::RichText::new("Reset to defaults")
                                .size(15.0)
                                .color(text_primary)
                        )
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(8.0)
                        .min_size(egui::vec2(140.0, 42.0));

                        if ui.add(reset_btn).clicked() {
                            self.settings = crate::settings::UserSettings::default();
                            self.sync_settings();
                            self.save_current_settings();
                            self.settings_saved_feedback = Some(time);
                            self.audio.play(SoundEffect::UiClick);
                        }

                        if let Some(t) = self.settings_saved_feedback {
                            if (time - t) < 2.5 || t > 1e8 {
                                ui.horizontal(|ui| {
                                    ui_tokens::checkmark_icon(ui, self.theme.colors.success(), 16.0);
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("Settings saved!").size(15.0).strong().color(self.theme.colors.success()));
                                });
                            }
                        }
                    });

                    ui.add_space(24.0);
                });
            });
        });
    }

    fn show_profile_cosmetics(&mut self, ui: &mut egui::Ui) {
        let card_bg = self.theme.colors.card_bg();
        let border = self.theme.colors.border();
        let text_primary = self.theme.colors.text_primary();
        let text_muted = self.theme.colors.text_muted();
        let accent = self.theme.colors.accent();
        let success = self.theme.colors.success();

        if ui_tokens::screen_header(
            ui,
            "PROFILE COSMETICS",
            text_primary,
            border,
            card_bg,
            |_| {},
        ) {
            self.audio.play(SoundEffect::UiClick);
            self.screen = AppScreen::Profiles;
        }

        ui.add_space(16.0);

        ui_tokens::centered_container(ui, 880.0, |ui| {
            let mut updated = false;
            if let Some(ref mut prof) = self.active_profile {
                ui.horizontal_top(|ui| {
                    // Column 1: Avatars & Badges Cards
                    ui.vertical(|ui| {
                        ui.set_width(260.0);

                        ui_tokens::card_frame(card_bg, border, 12.0, 16.0).show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Choose Avatar")
                                    .size(17.0)
                                    .strong()
                                    .color(text_primary),
                            );
                            ui.add_space(8.0);
                            let avatars = ["🧑", "👽", "🤖", "🎭", "💾", "⚡", "🔮", "🎮"];
                            egui::Grid::new("avatar_grid")
                                .spacing(egui::vec2(10.0, 10.0))
                                .show(ui, |ui| {
                                    for (idx, &avatar) in avatars.iter().enumerate() {
                                        let is_sel = prof.avatar_id == avatar;
                                        let btn = egui::Button::new(
                                            egui::RichText::new(avatar).size(26.0),
                                        )
                                        .fill(if is_sel { accent } else { card_bg })
                                        .stroke(if is_sel {
                                            egui::Stroke::NONE
                                        } else {
                                            egui::Stroke::new(1.0, border)
                                        })
                                        .corner_radius(8.0)
                                        .min_size(egui::vec2(44.0, 44.0));
                                        if ui.add(btn).clicked() {
                                            self.audio.play(SoundEffect::UiClick);
                                            prof.avatar_id = avatar.to_string();
                                            updated = true;
                                        }
                                        if (idx + 1) % 4 == 0 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        });

                        ui.add_space(14.0);

                        ui_tokens::card_frame(card_bg, border, 12.0, 16.0).show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Profile Badge")
                                    .size(17.0)
                                    .strong()
                                    .color(text_primary),
                            );
                            ui.add_space(8.0);
                            let badges = [
                                "None",
                                "🌟 Pioneer",
                                "🔥 Streak",
                                "🏆 Champion",
                                "⚔️ Veteran",
                            ];
                            for badge in badges {
                                let is_sel = prof.badge_id == badge;
                                let btn = egui::Button::new(
                                    egui::RichText::new(badge).size(14.0).strong().color(
                                        if is_sel {
                                            egui::Color32::WHITE
                                        } else {
                                            text_primary
                                        },
                                    ),
                                )
                                .fill(if is_sel { accent } else { card_bg })
                                .stroke(if is_sel {
                                    egui::Stroke::NONE
                                } else {
                                    egui::Stroke::new(1.0, border)
                                })
                                .corner_radius(8.0)
                                .min_size(egui::vec2(220.0, 32.0));
                                if ui.add(btn).clicked() {
                                    self.audio.play(SoundEffect::UiClick);
                                    prof.badge_id = badge.to_string();
                                    updated = true;
                                }
                                ui.add_space(4.0);
                            }
                        });
                    });

                    ui.add_space(20.0);

                    // Column 2: Titles & Preview Card
                    ui.vertical(|ui| {
                        let avail = ui.available_width();

                        ui_tokens::card_frame(card_bg, border, 12.0, 16.0).show(ui, |ui| {
                            ui.set_width(avail - 20.0);
                            ui.label(
                                egui::RichText::new("Player Title")
                                    .size(17.0)
                                    .strong()
                                    .color(text_primary),
                            );
                            ui.add_space(8.0);
                            let titles = [
                                "Novice Sweeper",
                                "Logic Master",
                                "Hexadecimal",
                                "Mine Specialist",
                                "The Untouchable",
                                "Circuit Breaker",
                            ];
                            for title in titles {
                                let is_sel = prof.title == title;
                                let btn = egui::Button::new(
                                    egui::RichText::new(title).size(15.0).color(if is_sel {
                                        egui::Color32::WHITE
                                    } else {
                                        text_primary
                                    }),
                                )
                                .fill(if is_sel { accent } else { card_bg })
                                .stroke(if is_sel {
                                    egui::Stroke::NONE
                                } else {
                                    egui::Stroke::new(1.0, border)
                                })
                                .corner_radius(8.0)
                                .min_size(egui::vec2(avail - 50.0, 34.0));
                                if ui.add(btn).clicked() {
                                    self.audio.play(SoundEffect::UiClick);
                                    prof.title = title.to_string();
                                    updated = true;
                                }
                                ui.add_space(6.0);
                            }
                        });

                        ui.add_space(14.0);

                        ui_tokens::card_frame(card_bg, border, 12.0, 20.0).show(ui, |ui| {
                            ui.set_width(avail - 20.0);
                            ui.label(
                                egui::RichText::new("Live Profile Card Preview")
                                    .size(17.0)
                                    .strong()
                                    .color(text_primary),
                            );
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&prof.avatar_id).size(52.0));
                                ui.add_space(14.0);
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&prof.display_name)
                                                .size(22.0)
                                                .strong()
                                                .color(text_primary),
                                        );
                                        ui.add_space(8.0);
                                        egui::Frame::new()
                                            .fill(accent)
                                            .corner_radius(8.0)
                                            .inner_margin(egui::vec2(8.0, 3.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Level {}",
                                                        prof.current_level
                                                    ))
                                                    .size(12.0)
                                                    .strong()
                                                    .color(egui::Color32::WHITE),
                                                );
                                            });
                                    });
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(&prof.title).size(15.0).color(accent),
                                    );
                                    if prof.badge_id != "None" && !prof.badge_id.is_empty() {
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(&prof.badge_id)
                                                .size(14.0)
                                                .strong()
                                                .color(success),
                                        );
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
                ui.label(
                    egui::RichText::new(
                        "No active profile selected. Select or create a profile first.",
                    )
                    .size(16.0)
                    .color(text_muted),
                );
            }
        });
    }
}

fn color_picker(ui: &mut eframe::egui::Ui, color: &mut (u8, u8, u8)) {
    let mut c = [color.0, color.1, color.2];
    ui.color_edit_button_srgb(&mut c);
    *color = (c[0], c[1], c[2]);
}

fn save_color_image_as_bmp(image: &egui::ColorImage, path: &str) -> std::io::Result<()> {
    use std::io::Write;
    let width = image.size[0] as u32;
    let height = image.size[1] as u32;
    let file_size = 54 + width * height * 4;

    let mut file = std::fs::File::create(path)?;
    file.write_all(b"BM")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&54u32.to_le_bytes())?;

    file.write_all(&40u32.to_le_bytes())?;
    file.write_all(&(width as i32).to_le_bytes())?;
    file.write_all(&(-(height as i32)).to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&32u16.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&(width * height * 4).to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;

    for pixel in &image.pixels {
        file.write_all(&[pixel.b(), pixel.g(), pixel.r(), pixel.a()])?;
    }
    file.flush()?;
    Ok(())
}

impl eframe::App for MineShiftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame_counter += 1;
        if let Some(ref path) = self.screenshot_target.clone() {
            if self.frame_counter == 5 {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            for event in ui.input(|i| i.raw.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    let _ = save_color_image_as_bmp(&image, path);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            if self.frame_counter > 100 {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        let time = ui.input(|i| i.time);
        let dt = if self.last_frame_time > 0.0 {
            (time - self.last_frame_time).clamp(0.001, 0.1) as f32
        } else {
            0.016
        };
        self.last_frame_time = time;

        self.shake.update(dt);
        self.particles.update(dt);
        ui.ctx().request_repaint();

        self.cursor_state = CustomCursorState::Default;
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if !matches!(self.screen, AppScreen::MainMenu) {
                if matches!(self.screen, AppScreen::Profiles) && self.active_profile.is_none() {
                    // Force selecting a profile
                } else {
                    self.audio.play(SoundEffect::UiClick);
                    self.screen = AppScreen::MainMenu;
                }
            }
        }

        // Dynamically synchronize egui visuals to active theme luminance and palette
        let mut visuals = if self.theme.is_dark() {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = self.theme.colors.bg();
        visuals.window_fill = self.theme.colors.bg();
        visuals.extreme_bg_color = self.theme.colors.card_bg();
        visuals.faint_bg_color = self.theme.colors.card_bg();
        visuals.widgets.noninteractive.bg_fill = self.theme.colors.button_bg();
        visuals.widgets.noninteractive.fg_stroke.color = self.theme.colors.text_primary();
        visuals.widgets.inactive.bg_fill = self.theme.colors.button_bg();
        visuals.widgets.inactive.fg_stroke.color = self.theme.colors.text_primary();
        visuals.widgets.hovered.bg_fill = self.theme.colors.button_bg();
        visuals.widgets.hovered.fg_stroke.color = self.theme.colors.accent();
        visuals.widgets.active.bg_fill = self.theme.colors.button_bg();
        visuals.widgets.active.fg_stroke.color = self.theme.colors.accent();
        visuals.selection.bg_fill = self.theme.colors.accent();
        visuals.slider_trailing_fill = true;
        visuals.handle_shape = egui::style::HandleShape::Circle;
        ui.ctx().set_visuals(visuals);

        let mut frame_style = egui::Frame::central_panel(ui.style());
        frame_style.fill = self.theme.colors.bg();

        egui::CentralPanel::default()
            .frame(frame_style)
            .show(ui, |ui| match self.screen {
                AppScreen::MainMenu => self.show_main_menu(ui),
                AppScreen::Gameplay => self.show_gameplay(ui),
                AppScreen::Profiles => self.show_profiles(ui),
                AppScreen::Statistics => self.show_statistics(ui),
                AppScreen::Achievements => self.show_achievements(ui),
                AppScreen::GameModes => self.show_game_modes(ui),
                AppScreen::MasteryChallenges => self.show_mastery_challenges(ui),
                AppScreen::SandboxLab => self.show_sandbox_lab(ui),
                AppScreen::GeneratingNoGuess => self.show_generating(ui),
                AppScreen::ThemeStudio => self.show_theme_studio(ui),
                AppScreen::ProfileCosmetics => self.show_profile_cosmetics(ui),
                AppScreen::Settings => self.show_settings(ui),
            });

        // 1. Render particle effects layer on top of gameplay
        let p_painter = ui.ctx().layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("particles_layer"),
        ));
        self.particles.render(&p_painter);

        // 2. Render premium custom cursor on highest tooltip layer
        let cursor_settings = &self.theme.cursor;
        if cursor_settings.enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            if let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos()) {
                let painter = ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Tooltip,
                    egui::Id::new("custom_cursor"),
                ));

                let is_clicking = ui.ctx().input(|i| i.pointer.any_down());
                let base_radius = cursor_settings.size * 8.0;
                let radius = if is_clicking {
                    base_radius * 0.8
                } else {
                    base_radius
                };

                let requested_icon = ui.ctx().output(|o| o.cursor_icon);

                let mut state = self.cursor_state;
                if state == CustomCursorState::Default {
                    if requested_icon == egui::CursorIcon::PointingHand {
                        state = CustomCursorState::UIHover;
                    } else if requested_icon == egui::CursorIcon::NotAllowed {
                        state = CustomCursorState::Disabled;
                    }
                }

                if matches!(self.screen, AppScreen::GeneratingNoGuess) {
                    state = CustomCursorState::Loading;
                } else if matches!(self.screen, AppScreen::Gameplay) {
                    if self.board.status == minecore::board::GameStatus::Won {
                        state = CustomCursorState::Victory;
                    } else if self.board.status == minecore::board::GameStatus::Lost {
                        state = CustomCursorState::Defeat;
                    }
                }

                // Trail effect
                if cursor_settings.trail && !cursor_settings.reduced_motion {
                    if let Some(interact_pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                        let trail_pos = pos - (pos - interact_pos) * 0.5;
                        painter.circle_filled(
                            trail_pos,
                            radius * 0.6,
                            self.theme.colors.accent().linear_multiply(0.3),
                        );
                    }
                }

                let (fill, stroke_col, stroke_width) = match state {
                    CustomCursorState::Default => {
                        (self.theme.colors.accent(), self.theme.colors.border(), 1.5)
                    }
                    CustomCursorState::UIHover => (
                        self.theme.colors.text_primary(),
                        self.theme.colors.accent(),
                        2.0,
                    ),
                    CustomCursorState::BoardHover => (
                        egui::Color32::from_white_alpha(100),
                        self.theme.colors.accent(),
                        2.0,
                    ),
                    CustomCursorState::BoardFlag => {
                        (self.theme.colors.danger(), self.theme.colors.border(), 1.5)
                    }
                    CustomCursorState::Disabled => (
                        self.theme.colors.text_muted(),
                        self.theme.colors.border(),
                        1.0,
                    ),
                    CustomCursorState::Loading => (
                        self.theme.colors.accent().linear_multiply(0.5),
                        self.theme.colors.border(),
                        1.0,
                    ),
                    CustomCursorState::Victory => {
                        (self.theme.colors.success(), egui::Color32::WHITE, 2.0)
                    }
                    CustomCursorState::Defeat => {
                        (self.theme.colors.danger(), egui::Color32::WHITE, 2.0)
                    }
                };

                painter.circle_filled(pos, radius, fill);
                painter.circle_stroke(
                    pos,
                    radius + 2.0,
                    egui::Stroke::new(stroke_width, stroke_col),
                );

                // Inner geometric detail based on state
                match state {
                    CustomCursorState::UIHover => {
                        painter.circle_filled(pos, radius * 0.5, self.theme.colors.bg());
                    }
                    CustomCursorState::BoardFlag => {
                        let rect = egui::Rect::from_center_size(pos, egui::vec2(radius, radius));
                        painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
                    }
                    CustomCursorState::Disabled => {
                        painter.line_segment(
                            [
                                pos - egui::vec2(radius, radius),
                                pos + egui::vec2(radius, radius),
                            ],
                            egui::Stroke::new(1.5, stroke_col),
                        );
                    }
                    _ => {
                        painter.circle_filled(pos, radius * 0.3, egui::Color32::WHITE);
                    }
                }
            }
        }
    }
}
