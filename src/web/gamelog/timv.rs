use crate::protos::timv::{death_event::DeathCause, DeathEvent};

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
            EventType::TimvDetectiveBody(_)
            | EventType::TimvPsychicReport(_)
            | EventType::TimvSharedPurchase(_)
            | EventType::TimvRampageGoal(_) => "list-group-item-primary",
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
        } else if let Some(event) = shared_purchase.get(event) {
            EventType::TimvSharedPurchase(event)
        } else if let Some(event) = rampage_goal.get(event) {
            EventType::TimvRampageGoal(event)
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
        match self.cause() {
            DeathCause::BUKKIT => self.last_damage_cause().get_damage_desc(),
            DeathCause::CLAYMORE => "Claymore",
            DeathCause::SUICIDE_BOMB => "Suicide Bomb",
            DeathCause::TRAITOR_TRAP => "Trap",
            DeathCause::CREEPER => "Creepers",
            DeathCause::WOLF => "Wolf",
            DeathCause::TESTER_BOMB => "Tester Bomb",
            DeathCause::CAT => "Cat",
            DeathCause::ENDER_CHEST => "Ender Chest",
            DeathCause::ZOMBIE => "Zombie",
            DeathCause::POISONOUS_WATER => "Poisonous Water",
            DeathCause::MAP_VOID => "Map Void",
            DeathCause::MAP_FEATURE => "Map Feature",
        }
    }
}
