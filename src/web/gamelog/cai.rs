use crate::protos::{self, gamelog::GameEvent};

use super::{event::EventType, GameLogExtension};

#[derive(Clone, Copy)]
pub struct CaiExtension {}

impl GameLogExtension for CaiExtension {
    fn get_box_color(&self, event: &super::EventType) -> &'static str {
        match event {
            EventType::CaiCatch(_) => "list-group-item-primary",
            EventType::CaiEscape(_) => "list-group-item-primary",
            EventType::CaiCapture(_) => "list-group-item-primary",
            EventType::CaiDeath(_) => "list-group-item-secondary",
            _ => "",
        }
    }

    fn parse_event(&self, event: &GameEvent) -> EventType {
        use protos::cai::exts::*;
        if let Some(event) = death.get(event) {
            EventType::CaiDeath(event)
        } else if let Some(event) = capture.get(event) {
            EventType::CaiCapture(event)
        } else if let Some(event) = catch.get(event) {
            EventType::CaiCatch(event)
        } else if let Some(event) = escape.get(event) {
            EventType::CaiEscape(event)
        } else {
            EventType::Unknown
        }
    }
}
