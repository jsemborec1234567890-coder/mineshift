use crate::board::Geometry;
use crate::rules::{
    AssistPolicy, CluePolicy, FirstClickPolicy, FlagPolicy, LogicGuarantee, Ruleset,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ChallengeCategory {
    PerfectPrecision,
    NoFlags,
    FastDeduction,
    HexSpecialist,
    ClueExpert,
    SurvivalEndurance,
    RestrictedResources,
}

impl ChallengeCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PerfectPrecision => "Perfect Precision",
            Self::NoFlags => "No Flags",
            Self::FastDeduction => "Fast Deduction",
            Self::HexSpecialist => "Hex Specialist",
            Self::ClueExpert => "Clue Expert",
            Self::SurvivalEndurance => "Survival Endurance",
            Self::RestrictedResources => "Restricted Resources",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum ChallengeTier {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Master,
}

impl ChallengeTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
            Self::Master => "Master",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::Beginner => "#50C878",
            Self::Intermediate => "#4A90E2",
            Self::Advanced => "#9B59B6",
            Self::Expert => "#E67E22",
            Self::Master => "#E74C3C",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasteryChallenge {
    pub id: String,
    pub title: String,
    pub category: ChallengeCategory,
    pub tier: ChallengeTier,
    pub description: String,
    pub ruleset: Ruleset,
    pub target_time_sec: Option<f64>,
    pub xp_reward: i64,
    pub badge_reward: Option<String>,
}

impl MasteryChallenge {
    pub fn catalog() -> Vec<MasteryChallenge> {
        vec![
            // Tier 1: Beginner
            MasteryChallenge {
                id: "chal_noflag_beg".to_string(),
                title: "Blind Intuition".to_string(),
                category: ChallengeCategory::NoFlags,
                tier: ChallengeTier::Beginner,
                description: "Solve a beginner square board with flagging completely disabled."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 9,
                    height: 9,
                    radius: 0,
                    mine_count: 10,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::NoFlags,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::CertifiedNoGuess,
                    assist_policy: AssistPolicy::Disallowed,
                    time_limit_sec: None,
                    lives: 1,
                },
                target_time_sec: Some(90.0),
                xp_reward: 75,
                badge_reward: Some("badge_noflag_novice".to_string()),
            },
            MasteryChallenge {
                id: "chal_res_beg".to_string(),
                title: "Tactical Rationing".to_string(),
                category: ChallengeCategory::RestrictedResources,
                tier: ChallengeTier::Beginner,
                description: "Beginner board with only 5 flags available for 10 mines.".to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 9,
                    height: 9,
                    radius: 0,
                    mine_count: 10,
                    first_click_policy: FirstClickPolicy::StandardSafe,
                    flag_policy: FlagPolicy::LimitedFlags { max_flags: 5 },
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::StandardProbabilistic,
                    assist_policy: AssistPolicy::Allowed,
                    time_limit_sec: None,
                    lives: 1,
                },
                target_time_sec: None,
                xp_reward: 60,
                badge_reward: None,
            },
            MasteryChallenge {
                id: "chal_hex_beg".to_string(),
                title: "Axial Initiation".to_string(),
                category: ChallengeCategory::HexSpecialist,
                tier: ChallengeTier::Beginner,
                description: "Clear a Radius 3 Hexagonal board (37 cells, 8 mines).".to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Hex,
                    width: 7,
                    height: 7,
                    radius: 3,
                    mine_count: 8,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::CertifiedNoGuess,
                    assist_policy: AssistPolicy::Allowed,
                    time_limit_sec: None,
                    lives: 1,
                },
                target_time_sec: Some(120.0),
                xp_reward: 80,
                badge_reward: Some("badge_hex_novice".to_string()),
            },
            // Tier 2: Intermediate
            MasteryChallenge {
                id: "chal_ring2_int".to_string(),
                title: "Second Harmonic".to_string(),
                category: ChallengeCategory::ClueExpert,
                tier: ChallengeTier::Intermediate,
                description: "Solve a 12x12 grid featuring extended Ring-2 distance clues."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 12,
                    height: 12,
                    radius: 0,
                    mine_count: 22,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::ExtendedRing2,
                    logic_guarantee: LogicGuarantee::StandardProbabilistic,
                    assist_policy: AssistPolicy::Allowed,
                    time_limit_sec: None,
                    lives: 1,
                },
                target_time_sec: Some(180.0),
                xp_reward: 120,
                badge_reward: Some("badge_ring2_adept".to_string()),
            },
            MasteryChallenge {
                id: "chal_fast_int".to_string(),
                title: "Clockwork Deduction".to_string(),
                category: ChallengeCategory::FastDeduction,
                tier: ChallengeTier::Intermediate,
                description: "Clear a 10x10 No-Guess board in under 45 seconds.".to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 10,
                    height: 10,
                    radius: 0,
                    mine_count: 15,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::CertifiedNoGuess,
                    assist_policy: AssistPolicy::Disallowed,
                    time_limit_sec: Some(45.0),
                    lives: 1,
                },
                target_time_sec: Some(45.0),
                xp_reward: 110,
                badge_reward: None,
            },
            MasteryChallenge {
                id: "chal_hex_int".to_string(),
                title: "Honeycomb Traverse".to_string(),
                category: ChallengeCategory::HexSpecialist,
                tier: ChallengeTier::Intermediate,
                description: "Medium Hexagonal grid (Radius 4, 61 cells, 16 mines) with 3 lives."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Hex,
                    width: 9,
                    height: 9,
                    radius: 4,
                    mine_count: 16,
                    first_click_policy: FirstClickPolicy::StandardSafe,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::StandardProbabilistic,
                    assist_policy: AssistPolicy::Allowed,
                    time_limit_sec: None,
                    lives: 3,
                },
                target_time_sec: None,
                xp_reward: 130,
                badge_reward: None,
            },
            // Tier 3: Advanced
            MasteryChallenge {
                id: "chal_noflag_adv".to_string(),
                title: "Absolute Discipline".to_string(),
                category: ChallengeCategory::NoFlags,
                tier: ChallengeTier::Advanced,
                description: "Intermediate 16x16 board (40 mines) solved with zero flags."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 16,
                    height: 16,
                    radius: 0,
                    mine_count: 40,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::NoFlags,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::CertifiedNoGuess,
                    assist_policy: AssistPolicy::Disallowed,
                    time_limit_sec: None,
                    lives: 1,
                },
                target_time_sec: Some(240.0),
                xp_reward: 200,
                badge_reward: Some("badge_noflag_master".to_string()),
            },
            MasteryChallenge {
                id: "chal_surv_adv".to_string(),
                title: "Gauntlet of Iron".to_string(),
                category: ChallengeCategory::SurvivalEndurance,
                tier: ChallengeTier::Advanced,
                description: "High density survival puzzle with 2 lives and tight 120s timer."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 14,
                    height: 14,
                    radius: 0,
                    mine_count: 36,
                    first_click_policy: FirstClickPolicy::StandardSafe,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::StandardProbabilistic,
                    assist_policy: AssistPolicy::Allowed,
                    time_limit_sec: Some(120.0),
                    lives: 2,
                },
                target_time_sec: Some(120.0),
                xp_reward: 190,
                badge_reward: None,
            },
            // Tier 4: Expert
            MasteryChallenge {
                id: "chal_hex_exp".to_string(),
                title: "Hexagonal Singularity".to_string(),
                category: ChallengeCategory::HexSpecialist,
                tier: ChallengeTier::Expert,
                description: "Large Hexagon (Radius 5, 91 cells, 28 mines) with Ring-2 clues."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Hex,
                    width: 11,
                    height: 11,
                    radius: 5,
                    mine_count: 28,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::ExtendedRing2,
                    logic_guarantee: LogicGuarantee::StandardProbabilistic,
                    assist_policy: AssistPolicy::Allowed,
                    time_limit_sec: None,
                    lives: 1,
                },
                target_time_sec: Some(300.0),
                xp_reward: 260,
                badge_reward: Some("badge_hex_grandmaster".to_string()),
            },
            MasteryChallenge {
                id: "chal_prec_exp".to_string(),
                title: "Faultless Operation".to_string(),
                category: ChallengeCategory::PerfectPrecision,
                tier: ChallengeTier::Expert,
                description: "Expert 24x16 board (65 mines) with certified No-Guess guarantee."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 24,
                    height: 16,
                    radius: 0,
                    mine_count: 65,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::Standard,
                    clue_policy: CluePolicy::StandardOnly,
                    logic_guarantee: LogicGuarantee::CertifiedNoGuess,
                    assist_policy: AssistPolicy::Disallowed,
                    time_limit_sec: Some(300.0),
                    lives: 1,
                },
                target_time_sec: Some(300.0),
                xp_reward: 300,
                badge_reward: Some("badge_precision_elite".to_string()),
            },
            // Tier 5: Master
            MasteryChallenge {
                id: "chal_omni_mas".to_string(),
                title: "Grandmaster Trial".to_string(),
                category: ChallengeCategory::PerfectPrecision,
                tier: ChallengeTier::Master,
                description: "The ultimate trial: 30x16 Expert, No Flags, Ring-2 Clues, 99 Mines."
                    .to_string(),
                ruleset: Ruleset {
                    topology: Geometry::Square,
                    width: 30,
                    height: 16,
                    radius: 0,
                    mine_count: 99,
                    first_click_policy: FirstClickPolicy::GuaranteedZeroOpening,
                    flag_policy: FlagPolicy::NoFlags,
                    clue_policy: CluePolicy::ExtendedRing2,
                    logic_guarantee: LogicGuarantee::StandardProbabilistic,
                    assist_policy: AssistPolicy::Disallowed,
                    time_limit_sec: Some(400.0),
                    lives: 1,
                },
                target_time_sec: Some(400.0),
                xp_reward: 500,
                badge_reward: Some("badge_grandmaster_ascendant".to_string()),
            },
        ]
    }

    pub fn find_by_id(id: &str) -> Option<MasteryChallenge> {
        Self::catalog().into_iter().find(|c| c.id == id)
    }
}
