use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub is_custom: bool,
    pub colors: Colors,
    pub geometry: Geometry,
    #[serde(default)]
    pub cursor: CursorSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CursorSettings {
    pub enabled: bool,
    pub size: f32,
    pub trail: bool,
    pub reduced_motion: bool,
}

impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            trail: false,
            reduced_motion: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Colors {
    pub bg: (u8, u8, u8),
    pub menu_bg: (u8, u8, u8),

    pub card_bg: (u8, u8, u8),

    pub border: (u8, u8, u8),
    pub covered: (u8, u8, u8),
    pub revealed: (u8, u8, u8),
    pub text_primary: (u8, u8, u8),
    pub text_muted: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub danger: (u8, u8, u8),
    pub success: (u8, u8, u8),
    pub button_bg: (u8, u8, u8),
    pub hover_highlight: (u8, u8, u8, u8),
    pub number_colors: Vec<(u8, u8, u8)>,
}


impl Colors {
    pub fn bg(&self) -> Color32 { Color32::from_rgb(self.bg.0, self.bg.1, self.bg.2) }
    pub fn menu_bg(&self) -> Color32 { Color32::from_rgb(self.menu_bg.0, self.menu_bg.1, self.menu_bg.2) }
    pub fn card_bg(&self) -> Color32 { Color32::from_rgb(self.card_bg.0, self.card_bg.1, self.card_bg.2) }
    pub fn border(&self) -> Color32 { Color32::from_rgb(self.border.0, self.border.1, self.border.2) }
    pub fn covered(&self) -> Color32 { Color32::from_rgb(self.covered.0, self.covered.1, self.covered.2) }
    pub fn revealed(&self) -> Color32 { Color32::from_rgb(self.revealed.0, self.revealed.1, self.revealed.2) }
    pub fn text_primary(&self) -> Color32 { Color32::from_rgb(self.text_primary.0, self.text_primary.1, self.text_primary.2) }
    pub fn text_muted(&self) -> Color32 { Color32::from_rgb(self.text_muted.0, self.text_muted.1, self.text_muted.2) }
    pub fn accent(&self) -> Color32 { Color32::from_rgb(self.accent.0, self.accent.1, self.accent.2) }
    pub fn danger(&self) -> Color32 { Color32::from_rgb(self.danger.0, self.danger.1, self.danger.2) }
    pub fn success(&self) -> Color32 { Color32::from_rgb(self.success.0, self.success.1, self.success.2) }
    pub fn button_bg(&self) -> Color32 { Color32::from_rgb(self.button_bg.0, self.button_bg.1, self.button_bg.2) }
    pub fn hover(&self) -> Color32 { Color32::from_rgba_unmultiplied(self.hover_highlight.0, self.hover_highlight.1, self.hover_highlight.2, self.hover_highlight.3) }
    pub fn number_color(&self, mines: usize) -> Color32 {
        if mines == 0 { return self.text_primary(); }
        let idx = (mines - 1).min(self.number_colors.len().saturating_sub(1));
        if idx < self.number_colors.len() {
            let c = self.number_colors[idx];
            Color32::from_rgb(c.0, c.1, c.2)
        } else {
            self.text_primary()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Geometry {
    pub square_radius: f32,
    pub cell_spacing: f32,
    pub border_thickness: f32,
    pub shadow_depth: f32,
    pub animation_speed: f32,
}

impl Theme {
    pub fn get_preset(id: &str) -> Option<Theme> {
        match id {
            "soft_circuit" => Some(Self::soft_circuit()),
            "obsidian" => Some(Self::obsidian()),
            "aurora" => Some(Self::aurora()),
            "paper" => Some(Self::paper()),
            "retro" => Some(Self::retro()),
            "neon_arcade" => Some(Self::neon_arcade()),
            _ => None
        }
    }
    pub fn presets() -> Vec<Self> {
        vec![
            Self::soft_circuit(),
            Self::obsidian(),
            Self::aurora(),
            Self::paper(),
            Self::retro(),
            Self::neon_arcade(),
        ]
    }

    pub fn soft_circuit() -> Self {
        Self {
            id: "soft_circuit".into(),
            name: "Soft Circuit".into(),
            is_custom: false,
            colors: Colors {
                bg: (238, 226, 255),
                menu_bg: (238, 226, 255),
                card_bg: (255, 255, 255),

                covered: (161, 160, 168),
                revealed: (255, 214, 168),
                text_primary: (48, 48, 56),
                text_muted: (161, 160, 168),
                accent: (121, 101, 190),
                danger: (232, 92, 96),
                success: (96, 205, 177),
                button_bg: (200, 195, 215),
                hover_highlight: (255, 255, 255, 100),
                border: (200, 200, 210),
                number_colors: vec![(80, 128, 192), (64, 160, 128), (192, 64, 64), (121, 101, 190), (128, 0, 0), (0, 128, 128), (0, 0, 0), (128, 128, 128)],



            },
            cursor: CursorSettings::default(),
            geometry: Geometry {
                square_radius: 6.0,
                cell_spacing: 4.0,
                border_thickness: 1.0,
                shadow_depth: 3.0,
                animation_speed: 1.0,
            },
        }
    }

    pub fn obsidian() -> Self {
        Self {
            id: "obsidian".into(),
            name: "Obsidian".into(),
            is_custom: false,
            colors: Colors {
                bg: (25, 25, 28),
                menu_bg: (25, 25, 28),
                card_bg: (36, 36, 42),

                covered: (45, 45, 50),
                revealed: (60, 60, 65),
                text_primary: (240, 240, 245),
                text_muted: (120, 120, 130),
                accent: (140, 100, 220),
                danger: (255, 80, 100),
                success: (80, 230, 160),
                button_bg: (40, 40, 45),
                hover_highlight: (200, 180, 255, 60),
                border: (60, 60, 70),
                number_colors: vec![(80, 128, 192), (64, 160, 128), (192, 64, 64), (121, 101, 190), (128, 0, 0), (0, 128, 128), (0, 0, 0), (128, 128, 128)],
            },
            cursor: CursorSettings::default(),
            geometry: Geometry {
                square_radius: 4.0,
                cell_spacing: 3.0,
                border_thickness: 1.5,
                shadow_depth: 4.0,
                animation_speed: 1.0,
            },
        }
    }

    pub fn aurora() -> Self {
        Self {
            id: "aurora".into(),
            name: "Aurora".into(),
            is_custom: false,
            colors: Colors {
                bg: (15, 30, 45),
                menu_bg: (15, 30, 45),
                card_bg: (24, 46, 68),

                covered: (30, 70, 95),
                revealed: (60, 120, 150),
                text_primary: (220, 245, 255),
                text_muted: (100, 160, 190),
                accent: (80, 230, 210),
                danger: (255, 100, 130),
                success: (100, 255, 150),
                button_bg: (40, 90, 120),
                hover_highlight: (150, 240, 255, 90),
                border: (45, 85, 120),
                number_colors: vec![(80, 128, 192), (64, 160, 128), (192, 64, 64), (121, 101, 190), (128, 0, 0), (0, 128, 128), (0, 0, 0), (128, 128, 128)],
            },
            cursor: CursorSettings::default(),
            geometry: Geometry {
                square_radius: 8.0,
                cell_spacing: 4.0,
                border_thickness: 0.0,
                shadow_depth: 2.0,
                animation_speed: 1.0,
            },
        }
    }

    pub fn paper() -> Self {
        Self {
            id: "paper".into(),
            name: "Paper".into(),
            is_custom: false,
            colors: Colors {
                bg: (248, 245, 240),
                menu_bg: (248, 245, 240),
                card_bg: (255, 254, 252),

                covered: (220, 215, 205),
                revealed: (250, 248, 245),
                text_primary: (40, 35, 30),
                text_muted: (150, 140, 130),
                accent: (180, 70, 60),
                danger: (200, 40, 40),
                success: (60, 160, 100),
                button_bg: (230, 225, 215),
                hover_highlight: (0, 0, 0, 20),
                border: (215, 210, 200),
                number_colors: vec![(80, 128, 192), (64, 160, 128), (192, 64, 64), (121, 101, 190), (128, 0, 0), (0, 128, 128), (0, 0, 0), (128, 128, 128)],
            },
            cursor: CursorSettings::default(),
            geometry: Geometry {
                square_radius: 2.0,
                cell_spacing: 3.0,
                border_thickness: 1.0,
                shadow_depth: 1.0,
                animation_speed: 1.0,
            },
        }
    }

    pub fn retro() -> Self {
        Self {
            id: "retro".into(),
            name: "Retro".into(),
            is_custom: false,
            colors: Colors {
                bg: (192, 192, 192),
                menu_bg: (192, 192, 192),
                card_bg: (214, 214, 214),

                covered: (192, 192, 192),
                revealed: (220, 220, 220),
                text_primary: (0, 0, 0),
                text_muted: (100, 100, 100),
                accent: (0, 0, 128),
                danger: (255, 0, 0),
                success: (0, 128, 0),
                button_bg: (192, 192, 192),
                hover_highlight: (255, 255, 255, 120),
                border: (128, 128, 128),
                number_colors: vec![(80, 128, 192), (64, 160, 128), (192, 64, 64), (121, 101, 190), (128, 0, 0), (0, 128, 128), (0, 0, 0), (128, 128, 128)],
            },
            cursor: CursorSettings::default(),
            geometry: Geometry {
                square_radius: 0.0,
                cell_spacing: 1.0,
                border_thickness: 2.0,
                shadow_depth: 0.0,
                animation_speed: 1.0,
            },
        }
    }

    pub fn neon_arcade() -> Self {
        Self {
            id: "neon_arcade".into(),
            name: "Neon Arcade".into(),
            is_custom: false,
            colors: Colors {
                bg: (10, 5, 20),
                menu_bg: (10, 5, 20),
                card_bg: (28, 14, 52),

                covered: (40, 20, 60),
                revealed: (70, 30, 90),
                text_primary: (255, 150, 255),
                text_muted: (150, 80, 150),
                accent: (255, 0, 255),
                danger: (255, 50, 50),
                success: (50, 255, 255),
                button_bg: (60, 20, 80),
                hover_highlight: (255, 100, 255, 150),
                border: (80, 20, 110),
                number_colors: vec![(80, 128, 192), (64, 160, 128), (192, 64, 64), (121, 101, 190), (128, 0, 0), (0, 128, 128), (0, 0, 0), (128, 128, 128)],
            },
            cursor: CursorSettings::default(),
            geometry: Geometry {
                square_radius: 12.0,
                cell_spacing: 6.0,
                border_thickness: 2.0,
                shadow_depth: 8.0,
                animation_speed: 1.0,
            },
        }
    }
}
