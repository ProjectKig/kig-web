use crate::protos::gamelog::GameEvent_oneof_extension;

use super::GameLogExtension;

use GameEvent_oneof_extension::*;

#[derive(Clone, Copy)]
pub struct TimvExtension {}

impl GameLogExtension for TimvExtension {
    fn get_box_color(&self, event: &super::WrappedEvent<'_>) -> &'static str {
        match event.get_raw_event() {
            Catch(_) => "list-group-item-primary",
            Escape(_) => "list-group-item-primary",
            Capture(_) => "list-group-item-primary",
            Death(_) => "list-group-item-secondary",
            _ => "",
        }
    }
}
