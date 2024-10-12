#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumString)]
#[strum(serialize_all = "UPPERCASE")]
pub enum GameMode {
    CAI,
    TIMV,
    BP,
    GRAV,
    BED,
    Halloween2023,
    Halloween2024,
}

impl GameMode {
    pub fn get_full_name(self) -> &'static str {
        match self {
            GameMode::CAI => "Cowboys and Indians",
            GameMode::TIMV => "Trouble in Mineville",
            GameMode::BP => "BlockParty",
            GameMode::GRAV => "Gravity",
            GameMode::BED => "Bed Wars",
            GameMode::Halloween2023 => "Kig-o'-ween (2023)",
            GameMode::Halloween2024 => "Kig-o'-ween (2024)",
        }
    }

    pub fn get_database_id(self) -> &'static str {
        match self {
            GameMode::CAI => "cai",
            GameMode::TIMV => "timv",
            GameMode::BP => "bp",
            GameMode::GRAV => "grav",
            GameMode::BED => "bed",
            GameMode::Halloween2023 => "halloween2023",
            GameMode::Halloween2024 => "halloween2024",
        }
    }
}
