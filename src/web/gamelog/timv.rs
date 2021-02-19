use super::{event::EventType, GameLogExtension};

#[derive(Clone, Copy)]
pub struct TimvExtension {}

impl GameLogExtension for TimvExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::CaiCatch(_) => "list-group-item-primary",
            EventType::CaiEscape(_) => "list-group-item-primary",
            EventType::CaiCapture(_) => "list-group-item-primary",
            EventType::CaiDeath(_) => "list-group-item-secondary",
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
        } else {
            EventType::Unknown
        }
    }
}
