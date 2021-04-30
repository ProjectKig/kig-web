use crate::protos::bp::{DeathEvent_PlayerDeathEvent, DeathEvent_PlayerDeathEvent_DeathCause};

use super::{event::EventType, GameLogExtension};

#[derive(Clone, Copy)]
pub struct BpExtension {}

impl GameLogExtension for BpExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::BpRound(_) => "list-group-item-primary",
            EventType::BpWinners(_) => "list-group-item-success",
            EventType::BpPowerup(_) => "list-group-item-warning",
            EventType::BpDeath(_) => "list-group-item-secondary",
            _ => "",
        }
    }

    fn parse_event(&self, event: &crate::protos::gamelog::GameEvent) -> EventType {
        use crate::protos::bp::exts::*;
        if let Some(event) = death.get(event) {
            EventType::BpDeath(event)
        } else if let Some(event) = round.get(event) {
            EventType::BpRound(event)
        } else if let Some(event) = powerup.get(event) {
            EventType::BpPowerup(event)
        } else if let Some(event) = winners.get(event) {
            EventType::BpWinners(event)
        } else {
            EventType::Unknown
        }
    }

    fn supports_score(&self) -> bool {
        false
    }
}

impl DeathEvent_PlayerDeathEvent {
    pub fn get_damage_desc(&self) -> &'static str {
        use DeathEvent_PlayerDeathEvent_DeathCause as DeathEvent_DeathCause;
        match self.get_death_cause() {
            DeathEvent_DeathCause::VOID => "Void",
            DeathEvent_DeathCause::DETECTION => "Void",
            DeathEvent_DeathCause::UNKNOWN => "Unknown cause",
        }
    }
}
