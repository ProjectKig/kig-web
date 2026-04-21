use super::{event::EventType, GameLogExtension};
use crate::protos::turf::{PhaseType, PhaseEvent};

#[derive(Clone, Copy)]
pub struct TurfExtension {}

impl GameLogExtension for TurfExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::TurfDeath(_) => "list-group-item-secondary",
            EventType::TurfPhase(_) => "list-group-item-primary",
            _ => "",
        }
    }

    fn parse_event(&self, event: &crate::protos::gamelog::GameEvent) -> EventType {
        use crate::protos::turf::exts::*;
        if let Some(event) = death.get(event) {
            EventType::TurfDeath(event)
        } else if let Some(event) = phase.get(event) {
            EventType::TurfPhase(event)
        } else {
            EventType::Unknown
        }
    }
}

impl PhaseEvent {
    pub fn name(&self) -> &'static str {
        match self.phase_type() {
            PhaseType::Build => "Build",
            PhaseType::Fight => "Fight",
        }
    }
}
