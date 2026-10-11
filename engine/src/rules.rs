use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FirstClickPolicy {
    StandardSafe,
    GuaranteedZeroOpening,
    ValidatedDeductionOpening,
}

impl Default for FirstClickPolicy {
    fn default() -> Self {
        Self::StandardSafe
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FlagPolicy {
    Standard,
    NoFlags,
    LimitedFlags { max_flags: usize },
}

impl Default for FlagPolicy {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CluePolicy {
    StandardOnly,
    ExtendedRing2,
}

impl Default for CluePolicy {
    fn default() -> Self {
        Self::StandardOnly
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LogicGuarantee {
    StandardProbabilistic,
    CertifiedNoGuess,
}

impl Default for LogicGuarantee {
    fn default() -> Self {
        Self::StandardProbabilistic
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AssistPolicy {
    Allowed,
    Disallowed,
}

impl Default for AssistPolicy {
    fn default() -> Self {
        Self::Allowed
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ruleset {
    pub topology: crate::board::Geometry,
    pub width: usize,
    pub height: usize,
    pub radius: usize,
    pub mine_count: usize,
    pub first_click_policy: FirstClickPolicy,
    pub flag_policy: FlagPolicy,
    pub clue_policy: CluePolicy,
    pub logic_guarantee: LogicGuarantee,
    pub assist_policy: AssistPolicy,
    pub time_limit_sec: Option<f64>,
    pub lives: usize,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self {
            topology: crate::board::Geometry::Square,
            width: 9,
            height: 9,
            radius: 0,
            mine_count: 10,
            first_click_policy: FirstClickPolicy::StandardSafe,
            flag_policy: FlagPolicy::Standard,
            clue_policy: CluePolicy::StandardOnly,
            logic_guarantee: LogicGuarantee::StandardProbabilistic,
            assist_policy: AssistPolicy::Allowed,
            time_limit_sec: None,
            lives: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesetError {
    InvalidDimensions(String),
    TooManyMines(String),
    ZeroMines,
    InvalidFlagLimit(String),
    ContradictoryConfiguration(String),
}

impl std::fmt::Display for RulesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimensions(msg) => write!(f, "Invalid dimensions: {}", msg),
            Self::TooManyMines(msg) => write!(f, "Too many mines: {}", msg),
            Self::ZeroMines => write!(f, "Mine count must be at least 1"),
            Self::InvalidFlagLimit(msg) => write!(f, "Invalid flag limit: {}", msg),
            Self::ContradictoryConfiguration(msg) => {
                write!(f, "Contradictory configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for RulesetError {}

impl Ruleset {
    pub fn classic_square(width: usize, height: usize, mine_count: usize) -> Self {
        Self {
            topology: crate::board::Geometry::Square,
            width,
            height,
            radius: 0,
            mine_count,
            ..Default::default()
        }
    }

    pub fn classic_hex(radius: usize, mine_count: usize) -> Self {
        let span = radius * 2 + 1;
        Self {
            topology: crate::board::Geometry::Hex,
            width: span,
            height: span,
            radius,
            mine_count,
            ..Default::default()
        }
    }

    pub fn total_active_cells(&self) -> usize {
        match self.topology {
            crate::board::Geometry::Square => self.width * self.height,
            crate::board::Geometry::Hex => 3 * self.radius * (self.radius + 1) + 1,
        }
    }

    pub fn validate(&self) -> Result<(), RulesetError> {
        if self.mine_count == 0 {
            return Err(RulesetError::ZeroMines);
        }

        match self.topology {
            crate::board::Geometry::Square => {
                if self.width < 3 || self.height < 3 {
                    return Err(RulesetError::InvalidDimensions(
                        "Board width and height must be >= 3".into(),
                    ));
                }
                if self.width > 50 || self.height > 50 {
                    return Err(RulesetError::InvalidDimensions(
                        "Board width and height must be <= 50".into(),
                    ));
                }
                let total = self.width * self.height;
                if self.mine_count >= total - 9 {
                    return Err(RulesetError::TooManyMines(format!(
                        "Mine count ({}) exceeds safe threshold for total cells ({})",
                        self.mine_count, total
                    )));
                }
            }
            crate::board::Geometry::Hex => {
                if self.radius < 2 || self.radius > 12 {
                    return Err(RulesetError::InvalidDimensions(
                        "Hex radius must be between 2 and 12".into(),
                    ));
                }
                let total = self.total_active_cells();
                if self.mine_count >= total - 7 {
                    return Err(RulesetError::TooManyMines(format!(
                        "Mine count ({}) exceeds safe threshold for total hex cells ({})",
                        self.mine_count, total
                    )));
                }
            }
        }

        if let FlagPolicy::LimitedFlags { max_flags } = self.flag_policy {
            if max_flags == 0 {
                return Err(RulesetError::InvalidFlagLimit(
                    "Limited flags limit must be >= 1 (use NoFlags instead)".into(),
                ));
            }
            if max_flags > self.mine_count * 2 {
                return Err(RulesetError::InvalidFlagLimit(
                    "Flag limit cannot exceed double the mine count".into(),
                ));
            }
        }

        if self.lives == 0 {
            return Err(RulesetError::ContradictoryConfiguration(
                "Lives must be at least 1".into(),
            ));
        }

        if let Some(t) = self.time_limit_sec {
            if t <= 0.0 {
                return Err(RulesetError::ContradictoryConfiguration(
                    "Time limit must be positive".into(),
                ));
            }
        }

        if self.logic_guarantee == LogicGuarantee::CertifiedNoGuess {
            let total = self.total_active_cells();
            if self.mine_count * 100 / total > 40 {
                return Err(RulesetError::ContradictoryConfiguration(
                    "Certified No Guess requires mine density <= 40%".into(),
                ));
            }
        }

        Ok(())
    }

    pub fn modifier_summary(&self) -> Vec<&'static str> {
        let mut mods = Vec::new();
        match self.flag_policy {
            FlagPolicy::NoFlags => mods.push("No Flags"),
            FlagPolicy::LimitedFlags { .. } => mods.push("Limited Flags"),
            FlagPolicy::Standard => {}
        }
        if self.clue_policy == CluePolicy::ExtendedRing2 {
            mods.push("Ring-2 Clues");
        }
        if self.logic_guarantee == LogicGuarantee::CertifiedNoGuess {
            mods.push("Certified No Guess");
        }
        if self.first_click_policy == FirstClickPolicy::GuaranteedZeroOpening {
            mods.push("Zero Opening");
        }
        if self.assist_policy == AssistPolicy::Disallowed {
            mods.push("Unassisted");
        }
        mods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruleset_validation_success() {
        let rs = Ruleset::classic_square(9, 9, 10);
        assert!(rs.validate().is_ok());

        let hex_rs = Ruleset::classic_hex(4, 15);
        assert!(hex_rs.validate().is_ok());
    }

    #[test]
    fn test_ruleset_validation_zero_mines() {
        let rs = Ruleset::classic_square(9, 9, 0);
        assert_eq!(rs.validate().unwrap_err(), RulesetError::ZeroMines);
    }

    #[test]
    fn test_ruleset_validation_invalid_dimensions() {
        let rs = Ruleset::classic_square(2, 2, 1);
        assert!(matches!(
            rs.validate(),
            Err(RulesetError::InvalidDimensions(_))
        ));

        let rs_too_big = Ruleset::classic_square(51, 10, 10);
        assert!(matches!(
            rs_too_big.validate(),
            Err(RulesetError::InvalidDimensions(_))
        ));

        let hex_bad = Ruleset::classic_hex(1, 1);
        assert!(matches!(
            hex_bad.validate(),
            Err(RulesetError::InvalidDimensions(_))
        ));
    }

    #[test]
    fn test_ruleset_validation_too_many_mines() {
        let rs = Ruleset::classic_square(9, 9, 75);
        assert!(matches!(rs.validate(), Err(RulesetError::TooManyMines(_))));

        let hex_rs = Ruleset::classic_hex(2, 14);
        assert!(matches!(
            hex_rs.validate(),
            Err(RulesetError::TooManyMines(_))
        ));
    }

    #[test]
    fn test_ruleset_validation_limited_flags() {
        let mut rs = Ruleset::classic_square(9, 9, 10);
        rs.flag_policy = FlagPolicy::LimitedFlags { max_flags: 0 };
        assert!(matches!(
            rs.validate(),
            Err(RulesetError::InvalidFlagLimit(_))
        ));

        rs.flag_policy = FlagPolicy::LimitedFlags { max_flags: 25 };
        assert!(matches!(
            rs.validate(),
            Err(RulesetError::InvalidFlagLimit(_))
        ));

        rs.flag_policy = FlagPolicy::LimitedFlags { max_flags: 10 };
        assert!(rs.validate().is_ok());
    }

    #[test]
    fn test_ruleset_validation_certified_no_guess_high_density() {
        let mut rs = Ruleset::classic_square(9, 9, 36); // 36 / 81 = 44.4% density
        rs.logic_guarantee = LogicGuarantee::CertifiedNoGuess;
        assert!(matches!(
            rs.validate(),
            Err(RulesetError::ContradictoryConfiguration(_))
        ));

        rs.mine_count = 20; // 20 / 81 = 24.6% density
        assert!(rs.validate().is_ok());
    }

    #[test]
    fn test_ruleset_modifier_summary() {
        let mut rs = Ruleset::classic_square(10, 10, 15);
        rs.flag_policy = FlagPolicy::NoFlags;
        rs.clue_policy = CluePolicy::ExtendedRing2;
        rs.first_click_policy = FirstClickPolicy::GuaranteedZeroOpening;
        rs.logic_guarantee = LogicGuarantee::CertifiedNoGuess;

        let summary = rs.modifier_summary();
        assert!(summary.contains(&"No Flags"));
        assert!(summary.contains(&"Ring-2 Clues"));
        assert!(summary.contains(&"Zero Opening"));
        assert!(summary.contains(&"Certified No Guess"));
    }
}
