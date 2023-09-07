use super::{event::EventType, GameLogExtension};

#[derive(Clone, Copy)]
pub struct HalloweenExtension {}

impl GameLogExtension for HalloweenExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::HalloweenDeath(_) => "list-group-item-secondary",
            _ => "",
        }
    }

    fn parse_event(&self, event: &crate::protos::gamelog::GameEvent) -> EventType {
        use crate::protos::halloween::exts::*;
        if let Some(event) = death.get(event) {
            EventType::HalloweenDeath(event)
        } else {
            EventType::Unknown
        }
    }

    fn supports_score(&self) -> bool {
        false
    }
}
