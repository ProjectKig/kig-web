use crate::protos::bp::death_event::{player_death_event::DeathCause, PlayerDeathEvent};

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

impl PlayerDeathEvent {
    pub fn get_damage_desc(&self) -> &'static str {
        match self.death_cause() {
            DeathCause::VOID => "Void",
            DeathCause::DETECTION => "Void",
            DeathCause::UNKNOWN => "Unknown cause",
        }
    }
}
