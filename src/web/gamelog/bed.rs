use crate::protos::gamelog::GameLog;
use super::{event::EventType, GameLogExtension, WrappedExtension};
use crate::protos::herd::exts::log_ext;
use crate::protos::herd::{DeathEvent, DeathEvent_DeathCause};

#[derive(Clone)]
pub struct BedExtension {
    respawn: bool,
}

impl BedExtension {
    pub fn new(log: &GameLog) -> BedExtension {
        BedExtension {
            respawn: log_ext.get(log).map(|l| l.get_respawn()).unwrap_or(false),
        }
    }

    pub fn is_respawn(&self) -> bool {
        self.respawn
    }
}

impl WrappedExtension {
    pub fn is_respawn(&self) -> bool {
        match self {
            WrappedExtension::Bed(bed) => bed.is_respawn(),
            _ => false
        }
    }
}

impl GameLogExtension for BedExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::HerdDeath(_) => "list-group-item-secondary",
            EventType::HerdElimination(_) => "list-group-item-warning",
            EventType::BedBedDestruction(_) => "list-group-item-primary",
            _ => "",
        }
    }

    fn parse_event(&self, event: &crate::protos::gamelog::GameEvent) -> EventType {
        use crate::protos::herd::exts::*;
        use crate::protos::bed::exts::*;

        if let Some(event) = death.get(event) {
            EventType::HerdDeath(event)
        } else if let Some(event) = elimination.get(event) {
            EventType::HerdElimination(event)
        } else if let Some(event) = bed_destroy.get(event) {
            EventType::BedBedDestruction(event)
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
            DeathEvent_DeathCause::OWNED_ENTITY => "Companion",
            DeathEvent_DeathCause::DISCONNECT => "Disconnected",
            DeathEvent_DeathCause::OUT_OF_MAP => "Out of Map",
            _ => self.get_last_damage_cause().get_damage_desc(),
        }
    }
}