use crate::protos::gamelog as log;
use crate::protos::{cai, timv};

pub enum EventType {
    Unknown,

    // Global events
    Chat(log::ChatEvent),
    Join(log::JoinEvent),
    Leave(log::LeaveEvent),

    // CAI events
    CaiDeath(cai::DeathEvent),
    CaiCapture(cai::CaptureEvent),
    CaiCatch(cai::CatchEvent),
    CaiEscape(cai::EscapeEvent),

    // TIMV events
    TimvDeath(timv::DeathEvent),
    TimvTest(timv::TestEvent),
    TimvBody(timv::BodyEvent),
    TimvTrap(timv::TrapEvent),
}
