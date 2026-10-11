#[cfg(test)]
mod tests {
    use crate::theme::Theme;
    use crate::settings::UserSettings;
    use crate::juice::TraumaShake;
    use crate::particles::ParticleSystem;
    use crate::audio::SoundEffect;

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

        let theme: Theme = serde_json::from_str(old_json).expect("Failed to deserialize old theme format");
        assert_eq!(theme.cursor.enabled, true);
        assert_eq!(theme.cursor.size, 1.0);
        assert_eq!(theme.cursor.trail, false);
        assert_eq!(theme.cursor.reduced_motion, false);
    }

    #[test]
    fn test_user_settings_defaults_and_empty_json() {
        let empty_json = "{}";
        let settings: UserSettings = serde_json::from_str(empty_json).expect("Default settings from empty JSON");
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
        let profile_id = db.create_profile("Audiophile", "avatar_01").expect("Create profile");

        let mut custom_settings = UserSettings::default();
        custom_settings.master_volume = 0.65;
        custom_settings.sfx_volume = 0.85;
        custom_settings.screen_shake = 1.75;
        custom_settings.cursor.size = 2.2;
        custom_settings.cursor.trail = true;

        let json = serde_json::to_string(&custom_settings).expect("Serialize");
        db.save_profile_settings(profile_id, &json).expect("Save settings");

        let loaded_json = db.get_profile_settings(profile_id).expect("Query settings").expect("Found settings");
        let loaded_settings: UserSettings = serde_json::from_str(&loaded_json).expect("Deserialize");

        assert_eq!(custom_settings, loaded_settings);
        assert!((loaded_settings.cursor.size - 2.2).abs() < 1e-4);
        assert_eq!(loaded_settings.cursor.trail, true);
        assert!((loaded_settings.screen_shake - 1.75).abs() < 1e-4);
    }

    #[test]
    fn test_theme_luminance_and_is_dark() {
        let soft_circuit = Theme::soft_circuit();
        assert!(!soft_circuit.is_dark(), "Soft Circuit must be identified as a light theme");

        let paper = Theme::paper();
        assert!(!paper.is_dark(), "Paper must be identified as a light theme");

        let obsidian = Theme::obsidian();
        assert!(obsidian.is_dark(), "Obsidian must be identified as a dark theme");

        let neon_arcade = Theme::neon_arcade();
        assert!(neon_arcade.is_dark(), "Neon Arcade must be identified as a dark theme");

        let aurora = Theme::aurora();
        assert!(aurora.is_dark(), "Aurora must be identified as a dark theme");

        let retro = Theme::retro();
        assert!(!retro.is_dark(), "Retro (Win95 grey 192,192,192) must be identified as a light theme");
    }

    #[test]
    fn test_profile_theme_sqlite_persistence() {
        let db = crate::db::Database::new_in_memory().expect("In memory database");
        let profile_id = db.create_profile("ThemeTester", "avatar_02").expect("Create profile");

        let initial_profile = db.get_profiles().expect("Profiles").into_iter().find(|p| p.id == profile_id).unwrap();
        assert_eq!(initial_profile.selected_theme_id, "soft_circuit");

        // Switch to obsidian
        db.set_profile_theme(profile_id, "obsidian").expect("Set theme to obsidian");
        let updated_profile = db.get_profiles().expect("Profiles").into_iter().find(|p| p.id == profile_id).unwrap();
        assert_eq!(updated_profile.selected_theme_id, "obsidian");

        // Switch to neon_arcade
        db.set_profile_theme(profile_id, "neon_arcade").expect("Set theme to neon_arcade");
        let updated_profile2 = db.get_profiles().expect("Profiles").into_iter().find(|p| p.id == profile_id).unwrap();
        assert_eq!(updated_profile2.selected_theme_id, "neon_arcade");

        // Switch to paper
        db.set_profile_theme(profile_id, "paper").expect("Set theme to paper");
        let updated_profile3 = db.get_profiles().expect("Profiles").into_iter().find(|p| p.id == profile_id).unwrap();
        assert_eq!(updated_profile3.selected_theme_id, "paper");
    }
}
