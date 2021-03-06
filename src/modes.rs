#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumString)]
pub enum GameMode {
    CAI,
    TIMV,
    BP,
    GRAV,
}

impl GameMode {
    pub fn get_full_name(self) -> &'static str {
        match self {
            GameMode::CAI => "Cowboys and Indians",
            GameMode::TIMV => "Trouble in Mineville",
            GameMode::BP => "BlockParty",
            GameMode::GRAV => "Gravity",
        }
    }

    pub fn get_database_id(self) -> &'static str {
        match self {
            GameMode::CAI => "cai",
            GameMode::TIMV => "timv",
            GameMode::BP => "bp",
            GameMode::GRAV => "grav",
        }
    }
}
