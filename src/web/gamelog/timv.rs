use crate::protos::timv::{DeathEvent, DeathEvent_DeathCause};

use super::{event::EventType, GameLogExtension};

#[derive(Clone, Copy)]
pub struct TimvExtension {}

impl GameLogExtension for TimvExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::TimvTest(_) => "list-group-item-primary",
            EventType::TimvTrap(_) => "list-group-item-danger",
            EventType::TimvBody(_) => "list-group-item-warning",
            EventType::TimvDeath(_) => "list-group-item-secondary",
            EventType::TimvDetectiveBody(_) | EventType::TimvPsychicReport(_) => {
                "list-group-item-primary"
            }
            _ => "",
        }
    }

    fn parse_event(&self, event: &crate::protos::gamelog::GameEvent) -> EventType {
        use crate::protos::timv::exts::*;
        if let Some(event) = death.get(event) {
            EventType::TimvDeath(event)
        } else if let Some(event) = test.get(event) {
            EventType::TimvTest(event)
        } else if let Some(event) = body.get(event) {
            EventType::TimvBody(event)
        } else if let Some(event) = trap.get(event) {
            EventType::TimvTrap(event)
        } else if let Some(event) = detective.get(event) {
            EventType::TimvDetectiveBody(event)
        } else if let Some(event) = psychic.get(event) {
            EventType::TimvPsychicReport(event)
        } else {
            EventType::Unknown
        }
    }

    fn supports_score(&self) -> bool {
        false
    }
}

impl DeathEvent {
    pub fn get_damage_desc(&self) -> &'static str {
        match self.get_cause() {
            DeathEvent_DeathCause::BUKKIT => self.get_last_damage_cause().get_damage_desc(),
            DeathEvent_DeathCause::CLAYMORE => "Claymore",
            DeathEvent_DeathCause::SUICIDE_BOMB => "Suicide Bomb",
            DeathEvent_DeathCause::TRAITOR_TRAP => "Trap",
            DeathEvent_DeathCause::CREEPER => "Creepers",
            DeathEvent_DeathCause::WOLF => "Wolf",
            DeathEvent_DeathCause::TESTER_BOMB => "Tester Bomb",
            DeathEvent_DeathCause::CAT => "Cat",
            DeathEvent_DeathCause::ENDER_CHEST => "Ender Chest",
            DeathEvent_DeathCause::ZOMBIE => "Zombie",
        }
    }
}
