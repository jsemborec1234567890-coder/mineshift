#[cfg(test)]
mod tests {
    use crate::audio::SoundEffect;
    use crate::juice::TraumaShake;
    use crate::particles::ParticleSystem;
    use crate::settings::UserSettings;
    use crate::theme::Theme;

    #[test]
    fn test_older_theme_format_deserialization() {
        let old_json = r#"{
            "id": "old_theme",
            "name": "Old Theme",
            "is_custom": false,
            "colors": {
                "bg": [0,0,0],
                "menu_bg": [0,0,0],
                "card_bg": [0,0,0],
                "border": [0,0,0],
                "covered": [0,0,0],
                "revealed": [0,0,0],
                "text_primary": [0,0,0],
                "text_muted": [0,0,0],
                "accent": [0,0,0],
                "danger": [0,0,0],
                "success": [0,0,0],
                "button_bg": [0,0,0],
                "hover_highlight": [0,0,0,0],
                "number_colors": []
            },
            "geometry": {
                "cell_spacing": 1.0,
                "square_radius": 1.0,
                "shadow_depth": 1.0,
                "animation_speed": 1.0,
                "border_thickness": 1.0
            }
        }"#;

        let theme: Theme =
            serde_json::from_str(old_json).expect("Failed to deserialize old theme format");
        assert_eq!(theme.cursor.enabled, true);
        assert_eq!(theme.cursor.size, 1.0);
        assert_eq!(theme.cursor.trail, false);
        assert_eq!(theme.cursor.reduced_motion, false);
    }

    #[test]
    fn test_user_settings_defaults_and_empty_json() {
        let empty_json = "{}";
        let settings: UserSettings =
            serde_json::from_str(empty_json).expect("Default settings from empty JSON");
        assert!((settings.master_volume - 0.8).abs() < 1e-4);
        assert!((settings.sfx_volume - 0.9).abs() < 1e-4);
        assert!((settings.music_volume - 0.65).abs() < 1e-4);
        assert_eq!(settings.mute_audio, false);
        assert_eq!(settings.pitch_variation, true);
        assert!((settings.screen_shake - 1.0).abs() < 1e-4);
        assert_eq!(settings.particles_enabled, true);
        assert_eq!(settings.cursor.enabled, true);
    }

    #[test]
    fn test_user_settings_roundtrip() {
        let mut original = UserSettings::default();
        original.master_volume = 0.42;
        original.sfx_volume = 0.77;
        original.music_volume = 0.55;
        original.mute_audio = true;
        original.screen_shake = 1.85;
        original.cursor.size = 2.4;
        original.cursor.reduced_motion = true;

        let json = serde_json::to_string(&original).expect("Serialize settings");
        let parsed: UserSettings = serde_json::from_str(&json).expect("Deserialize settings");

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_trauma_shake_decay_and_bounds() {
        let mut shake = TraumaShake::new();
        assert_eq!(shake.trauma, 0.0);

        shake.add_trauma(0.5);
        assert!((shake.trauma - 0.5).abs() < 1e-4);

        // Clamps at 1.0
        shake.add_trauma(0.8);
        assert!((shake.trauma - 1.0).abs() < 1e-4);

        // Decays smoothly
        shake.update(0.2);
        assert!(shake.trauma < 1.0);
        assert!(shake.trauma > 0.0);

        // Decays to 0
        shake.update(2.0);
        assert_eq!(shake.trauma, 0.0);
    }

    #[test]
    fn test_particle_system_lifecycle() {
        let mut ps = ParticleSystem::new();
        assert!(ps.particles.is_empty());

        let center = eframe::egui::pos2(100.0, 100.0);
        let color = eframe::egui::Color32::RED;
        ps.spawn_explosion(center, color, false);
        assert!(!ps.particles.is_empty());

        let initial_count = ps.particles.len();
        // Update forwards
        ps.update(0.1);
        assert_eq!(ps.particles.len(), initial_count);

        // Fast forward past lifetime (particles live at most ~0.85s)
        for _ in 0..20 {
            ps.update(0.1);
        }
        assert!(ps.particles.is_empty());
    }

    #[test]
    fn test_all_sound_effects_instantiation() {
        let effects = [
            SoundEffect::TileClick,
            SoundEffect::TileReveal(0),
            SoundEffect::TileReveal(8),
            SoundEffect::FlagPlace,
            SoundEffect::FlagRemove,
            SoundEffect::Chord,
            SoundEffect::Explosion,
            SoundEffect::Victory,
            SoundEffect::Defeat,
            SoundEffect::UiClick,
        ];
        assert_eq!(effects.len(), 10);
    }

    #[test]
    fn test_sqlite_settings_persistence() {
        let db = crate::db::Database::new_in_memory().expect("In memory database");
        let profile_id = db
            .create_profile("Audiophile", "avatar_01")
            .expect("Create profile");

        let mut custom_settings = UserSettings::default();
        custom_settings.master_volume = 0.65;
        custom_settings.sfx_volume = 0.85;
        custom_settings.screen_shake = 1.75;
        custom_settings.cursor.size = 2.2;
        custom_settings.cursor.trail = true;

        let json = serde_json::to_string(&custom_settings).expect("Serialize");
        db.save_profile_settings(profile_id, &json)
            .expect("Save settings");

        let loaded_json = db
            .get_profile_settings(profile_id)
            .expect("Query settings")
            .expect("Found settings");
        let loaded_settings: UserSettings =
            serde_json::from_str(&loaded_json).expect("Deserialize");

        assert_eq!(custom_settings, loaded_settings);
        assert!((loaded_settings.cursor.size - 2.2).abs() < 1e-4);
        assert_eq!(loaded_settings.cursor.trail, true);
        assert!((loaded_settings.screen_shake - 1.75).abs() < 1e-4);
    }

    #[test]
    fn test_theme_luminance_and_is_dark() {
        let soft_circuit = Theme::soft_circuit();
        assert!(
            !soft_circuit.is_dark(),
            "Soft Circuit must be identified as a light theme"
        );

        let paper = Theme::paper();
        assert!(
            !paper.is_dark(),
            "Paper must be identified as a light theme"
        );

        let obsidian = Theme::obsidian();
        assert!(
            obsidian.is_dark(),
            "Obsidian must be identified as a dark theme"
        );

        let neon_arcade = Theme::neon_arcade();
        assert!(
            neon_arcade.is_dark(),
            "Neon Arcade must be identified as a dark theme"
        );

        let aurora = Theme::aurora();
        assert!(
            aurora.is_dark(),
            "Aurora must be identified as a dark theme"
        );

        let retro = Theme::retro();
        assert!(
            !retro.is_dark(),
            "Retro (Win95 grey 192,192,192) must be identified as a light theme"
        );
    }

    #[test]
    fn test_profile_theme_sqlite_persistence() {
        let db = crate::db::Database::new_in_memory().expect("In memory database");
        let profile_id = db
            .create_profile("ThemeTester", "avatar_02")
            .expect("Create profile");

        let initial_profile = db
            .get_profiles()
            .expect("Profiles")
            .into_iter()
            .find(|p| p.id == profile_id)
            .unwrap();
        assert_eq!(initial_profile.selected_theme_id, "soft_circuit");

        // Switch to obsidian
        db.set_profile_theme(profile_id, "obsidian")
            .expect("Set theme to obsidian");
        let updated_profile = db
            .get_profiles()
            .expect("Profiles")
            .into_iter()
            .find(|p| p.id == profile_id)
            .unwrap();
        assert_eq!(updated_profile.selected_theme_id, "obsidian");

        // Switch to neon_arcade
        db.set_profile_theme(profile_id, "neon_arcade")
            .expect("Set theme to neon_arcade");
        let updated_profile2 = db
            .get_profiles()
            .expect("Profiles")
            .into_iter()
            .find(|p| p.id == profile_id)
            .unwrap();
        assert_eq!(updated_profile2.selected_theme_id, "neon_arcade");

        // Switch to paper
        db.set_profile_theme(profile_id, "paper")
            .expect("Set theme to paper");
        let updated_profile3 = db
            .get_profiles()
            .expect("Profiles")
            .into_iter()
            .find(|p| p.id == profile_id)
            .unwrap();
        assert_eq!(updated_profile3.selected_theme_id, "paper");
    }

    #[test]
    fn test_font_definitions_windows_fallback() {
        let ctx = eframe::egui::Context::default();
        let mut fonts = eframe::egui::FontDefinitions::default();
        if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
            fonts.font_data.insert(
                "segoe_ui_symbol".to_owned(),
                std::sync::Arc::new(eframe::egui::FontData::from_owned(bytes)),
            );
            if let Some(prop) = fonts
                .families
                .get_mut(&eframe::egui::FontFamily::Proportional)
            {
                prop.push("segoe_ui_symbol".to_owned());
            }
        }
        ctx.set_fonts(fonts);
        // Ensure context can run and layout symbols without panic
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.label("✓ Checkmark");
            ui.label("🏆 Trophy");
            ui.label("⏱ Timer");
            ui.label("Settings saved!");
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_centered_container_preserves_vertical_space() {
        let ctx = eframe::egui::Context::default();
        let mut avail_h_inside = 0.0;
        let mut avail_w_inside = 0.0;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(eframe::egui::vec2(1280.0, 720.0));
            crate::ui_tokens::centered_container(ui, 960.0, |ui| {
                avail_w_inside = ui.available_width();
                avail_h_inside = ui.available_height();
            });
        });
        output.textures_delta.clear();
        assert_eq!(
            avail_w_inside, 960.0,
            "Centered container must allocate exactly max_width"
        );
        assert!(
            avail_h_inside > 600.0,
            "Centered container must not collapse vertical height (was {})",
            avail_h_inside
        );
    }

    #[test]
    fn test_theme_card_and_button_contrast() {
        let themes = [
            Theme::soft_circuit(),
            Theme::obsidian(),
            Theme::aurora(),
            Theme::paper(),
            Theme::retro(),
            Theme::neon_arcade(),
        ];
        for t in themes {
            let card = t.colors.card_bg();
            let button = t.colors.button_bg();
            assert_ne!(
                card, button,
                "Theme '{}' must have distinct card_bg and button_bg colors for slider track contrast",
                t.name
            );
        }
    }

    #[test]
    fn test_screen_shake_zero_vs_nonzero() {
        let mut shake = TraumaShake::new();
        // At 0% intensity
        shake.intensity = 0.0;
        shake.add_trauma(0.85);
        assert_eq!(shake.trauma, 0.0, "At 0% intensity, trauma must remain 0");
        assert_eq!(shake.get_offset(1.0), eframe::egui::Vec2::ZERO);

        // At 50% intensity
        shake.intensity = 0.5;
        shake.add_trauma(0.85);
        assert!(
            (shake.trauma - 0.425).abs() < 1e-4,
            "At 50% intensity, trauma must scale proportionally"
        );
        assert_ne!(
            shake.get_offset(1.0),
            eframe::egui::Vec2::ZERO,
            "At 50% intensity, offset must be nonzero"
        );

        // At 100% intensity
        shake.trauma = 0.0;
        shake.intensity = 1.0;
        shake.add_trauma(0.85);
        assert!(
            (shake.trauma - 0.85).abs() < 1e-4,
            "At 100% intensity, trauma must be 0.85"
        );
        assert_ne!(
            shake.get_offset(1.0),
            eframe::egui::Vec2::ZERO,
            "At 100% intensity, offset must be nonzero"
        );
    }

    #[test]
    fn test_bmp_exporter() {
        let img = eframe::egui::ColorImage::new(
            [10, 10],
            vec![eframe::egui::Color32::from_rgb(255, 0, 128); 100],
        );
        let mut buf = Vec::new();
        use std::io::Write;
        let width = img.size[0] as u32;
        let height = img.size[1] as u32;
        let file_size = 54 + width * height * 4;
        buf.write_all(b"BM").unwrap();
        buf.write_all(&file_size.to_le_bytes()).unwrap();
        buf.write_all(&0u32.to_le_bytes()).unwrap();
        buf.write_all(&54u32.to_le_bytes()).unwrap();
        buf.write_all(&40u32.to_le_bytes()).unwrap();
        buf.write_all(&(width as i32).to_le_bytes()).unwrap();
        buf.write_all(&(-(height as i32)).to_le_bytes()).unwrap();
        buf.write_all(&1u16.to_le_bytes()).unwrap();
        buf.write_all(&32u16.to_le_bytes()).unwrap();
        buf.write_all(&0u32.to_le_bytes()).unwrap();
        buf.write_all(&(width * height * 4).to_le_bytes()).unwrap();
        buf.write_all(&0u32.to_le_bytes()).unwrap();
        buf.write_all(&0u32.to_le_bytes()).unwrap();
        buf.write_all(&0u32.to_le_bytes()).unwrap();
        buf.write_all(&0u32.to_le_bytes()).unwrap();
        for pixel in &img.pixels {
            buf.write_all(&[pixel.b(), pixel.g(), pixel.r(), pixel.a()])
                .unwrap();
        }
        assert_eq!(buf.len(), 54 + 400);
        assert_eq!(&buf[0..2], b"BM");
    }

    #[test]
    fn test_mastery_challenge_catalog_validity() {
        use minecore::challenge::MasteryChallenge;
        let catalog = MasteryChallenge::catalog();
        assert_eq!(
            catalog.len(),
            11,
            "Catalog should have 11 mastery challenges across 5 tiers"
        );

        for c in &catalog {
            assert!(!c.id.is_empty(), "Challenge ID cannot be empty");
            assert!(!c.title.is_empty(), "Challenge title cannot be empty");
            assert!(
                !c.description.is_empty(),
                "Challenge description cannot be empty"
            );
            assert!(c.xp_reward > 0, "Challenge XP reward must be positive");
            assert!(
                c.ruleset.mine_count > 0,
                "Challenge mine count must be positive"
            );
        }
    }

    #[test]
    fn test_cli_args_mastery_and_sandbox_routing() {
        use crate::{AppScreen, MineShiftApp};
        let mut app = MineShiftApp::new();

        app.apply_cli_args(&[
            "mineshift".to_string(),
            "--screen".to_string(),
            "mastery".to_string(),
        ]);
        assert!(matches!(app.screen, AppScreen::MasteryChallenges));

        app.apply_cli_args(&[
            "mineshift".to_string(),
            "--screen".to_string(),
            "sandbox".to_string(),
        ]);
        assert!(matches!(app.screen, AppScreen::SandboxLab));

        app.apply_cli_args(&[
            "mineshift".to_string(),
            "--challenge".to_string(),
            "chal_noflag_beg".to_string(),
        ]);
        assert!(matches!(app.screen, AppScreen::Gameplay));
        assert_eq!(app.active_challenge_id, Some("chal_noflag_beg".to_string()));
        assert_eq!(
            app.board.ruleset.flag_policy,
            minecore::rules::FlagPolicy::NoFlags
        );

        app.apply_cli_args(&[
            "mineshift".to_string(),
            "--clue-policy".to_string(),
            "ring2".to_string(),
        ]);
        assert_eq!(
            app.board.ruleset.clue_policy,
            minecore::rules::CluePolicy::ExtendedRing2
        );
    }

    #[test]
    fn test_challenge_persistence_idempotency() {
        use crate::db::Database;
        let db = Database::new_in_memory().unwrap();
        let pid = db.create_profile("Challenger", "🤖").unwrap();

        // First win awards XP
        let first_win = db
            .record_challenge_completion(pid, "chal_noflag_beg", 45000, 75)
            .unwrap();
        assert!(first_win, "First win must return true and award XP");

        let profs = db.get_profiles().unwrap();
        let p = profs.into_iter().find(|p| p.id == pid).unwrap();
        assert_eq!(p.total_xp, 75);

        // Second faster win updates best time but does NOT double-award XP
        let second_win = db
            .record_challenge_completion(pid, "chal_noflag_beg", 38000, 75)
            .unwrap();
        assert!(!second_win, "Subsequent win must return false for XP award");

        let profs = db.get_profiles().unwrap();
        let p = profs.into_iter().find(|p| p.id == pid).unwrap();
        assert_eq!(p.total_xp, 75, "Total XP should not be double-awarded");

        let completions = db.get_challenge_completions(pid).unwrap();
        assert_eq!(completions.len(), 1);
        let c = completions.get("chal_noflag_beg").unwrap();
        assert_eq!(c.best_time_ms, 38000);
        assert_eq!(c.attempts, 2);
    }

    #[test]
    fn test_challenge_profile_isolation() {
        use crate::db::Database;
        let db = Database::new_in_memory().unwrap();
        let pid1 = db.create_profile("PlayerOne", "🦁").unwrap();
        let pid2 = db.create_profile("PlayerTwo", "🐯").unwrap();

        // Player 1 completes challenge
        db.record_challenge_completion(pid1, "chal_noflag_beg", 42000, 75)
            .unwrap();

        let comps1 = db.get_challenge_completions(pid1).unwrap();
        let comps2 = db.get_challenge_completions(pid2).unwrap();

        assert_eq!(comps1.len(), 1, "Player 1 should have 1 completion");
        assert_eq!(
            comps2.len(),
            0,
            "Player 2 must be strictly isolated with 0 completions"
        );

        let profs = db.get_profiles().unwrap();
        let p1 = profs.iter().find(|p| p.id == pid1).unwrap();
        let p2 = profs.iter().find(|p| p.id == pid2).unwrap();
        assert_eq!(p1.total_xp, 75);
        assert_eq!(p2.total_xp, 0);
    }

    #[test]
    fn test_challenge_attempt_increment_before_win() {
        use crate::db::Database;
        let db = Database::new_in_memory().unwrap();
        let pid = db.create_profile("TryHard", "🚀").unwrap();

        // 3 failed attempts
        db.record_challenge_attempt(pid, "chal_hard").unwrap();
        db.record_challenge_attempt(pid, "chal_hard").unwrap();
        db.record_challenge_attempt(pid, "chal_hard").unwrap();

        let comps = db.get_challenge_completions(pid).unwrap();
        let rec = comps.get("chal_hard").unwrap();
        assert_eq!(rec.attempts, 3);
        assert_eq!(rec.best_time_ms, 0, "No completion time recorded yet");

        // 4th attempt is a win
        db.record_challenge_completion(pid, "chal_hard", 50000, 100)
            .unwrap();
        let comps_after = db.get_challenge_completions(pid).unwrap();
        let rec_after = comps_after.get("chal_hard").unwrap();
        assert_eq!(rec_after.attempts, 4);
        assert_eq!(rec_after.best_time_ms, 50000);
    }

    #[test]
    fn test_nine_game_modes_invariants() {
        use minecore::mode::{GameModeId, ModeState};
        let modes = [
            GameModeId::Classic,
            GameModeId::Sprint,
            GameModeId::NoGuess,
            GameModeId::Blitz,
            GameModeId::Survival,
            GameModeId::Daily,
            GameModeId::Expedition,
            GameModeId::Zen,
            GameModeId::Sandbox,
        ];
        assert_eq!(modes.len(), 9);

        for m in modes {
            assert!(!m.name().is_empty());
            assert!(!m.desc().is_empty());

            let state = ModeState::new(m);
            assert_eq!(state.id, m);
            assert_eq!(state.score, 0);
            assert!(state.lives >= 1);

            if m == GameModeId::Survival {
                assert_eq!(state.lives, 3, "Survival mode must start with 3 lives");
            }
            if m == GameModeId::Blitz {
                assert_eq!(
                    state.time_limit_sec,
                    Some(180.0),
                    "Blitz mode must start with 180s timer"
                );
            }
        }
    }

    #[test]
    fn test_performance_stress_suite() {
        use crate::db::Database;
        use minecore::board::Board;
        use minecore::no_guess::generate_no_guess_board;
        use minecore::solver::Solver;
        use std::time::Instant;

        // 1. Repeated Board Creation (1,000 boards)
        let t0 = Instant::now();
        for _ in 0..1000 {
            let _ = Board::new_square(16, 16, 40);
            let _ = Board::new_hex(5, 20);
        }
        let create_elapsed = t0.elapsed();
        println!("1,000 Square + Hex board creations: {:?}", create_elapsed);
        assert!(
            create_elapsed.as_millis() < 500,
            "1,000 creations must execute under 500ms"
        );

        // 2. Large Flood Reveal (40x40 board, 1 mine at corner, click opposite corner)
        let mut large_board = Board::new_square(40, 40, 0);
        large_board.first_click = false;
        let mine_idx = large_board.get_index(0, 0);
        large_board.cells[mine_idx].is_mine = true;
        let t_flood = Instant::now();
        large_board.reveal(39, 39);
        let flood_elapsed = t_flood.elapsed();
        println!("40x40 1,599 cell flood reveal: {:?}", flood_elapsed);
        assert!(
            flood_elapsed.as_millis() < 50,
            "Large flood reveal must execute under 50ms"
        );

        // 3. Solver deduction throughput on a realistic 16x16 board
        let mut sim_board = Board::new_square(16, 16, 30);
        sim_board.reveal(8, 8);
        let t_solve = Instant::now();
        for _ in 0..50 {
            let _ = Solver::deduce(&sim_board);
            let _ = Solver::explain_step(&sim_board);
        }
        let solve_elapsed = t_solve.elapsed();
        println!(
            "50 Solver deduction + explanation cycles: {:?}",
            solve_elapsed
        );
        assert!(
            solve_elapsed.as_millis() < 100,
            "50 solver cycles must execute under 100ms"
        );

        // 4. Repeated No Guess board generation (5 square + 5 hex)
        let t_ng = Instant::now();
        for _ in 0..5 {
            let b_sq = Board::new_square(6, 6, 4);
            let _ = generate_no_guess_board(b_sq, 2, 2);
            let b_hex = Board::new_hex(2, 3);
            let _ = generate_no_guess_board(b_hex, 2, 2);
        }
        let ng_elapsed = t_ng.elapsed();
        println!("10 No Guess board generations: {:?}", ng_elapsed);
        assert!(
            ng_elapsed.as_millis() < 1000,
            "10 No Guess generations must execute under 1000ms"
        );

        // 5. SQLite database transactions (100 operations)
        let db = Database::new_in_memory().unwrap();
        let pid = db.create_profile("StressUser", "⚡").unwrap();
        let t_db = Instant::now();
        for _ in 0..100 {
            let _ = db.record_challenge_attempt(pid, "chal_test");
            let _ = db.add_stat(pid, "total_clicks", 1);
        }
        let db_elapsed = t_db.elapsed();
        println!("200 SQLite stat + attempt writes: {:?}", db_elapsed);
        assert!(
            db_elapsed.as_millis() < 100,
            "200 DB writes must execute under 100ms"
        );
    }
}
