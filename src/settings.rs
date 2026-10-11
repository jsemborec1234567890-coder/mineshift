use serde::{Deserialize, Serialize};
use crate::theme::CursorSettings;

fn default_vol() -> f32 {
    0.8
}

fn default_sfx_vol() -> f32 {
    0.9
}

fn default_shake() -> f32 {
    1.0
}

fn default_speed() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserSettings {
    #[serde(default)]
    pub cursor: CursorSettings,

    #[serde(default = "default_vol")]
    pub master_volume: f32,

    #[serde(default = "default_sfx_vol")]
    pub sfx_volume: f32,

    #[serde(default)]
    pub mute_audio: bool,

    #[serde(default = "default_true")]
    pub pitch_variation: bool,

    #[serde(default = "default_shake")]
    pub screen_shake: f32,

    #[serde(default = "default_true")]
    pub particles_enabled: bool,

    #[serde(default = "default_speed")]
    pub animation_speed: f32,

    #[serde(default)]
    pub high_contrast: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            cursor: CursorSettings::default(),
            master_volume: 0.8,
            sfx_volume: 0.9,
            mute_audio: false,
            pitch_variation: true,
            screen_shake: 1.0,
            particles_enabled: true,
            animation_speed: 1.0,
            high_contrast: false,
        }
    }
}
