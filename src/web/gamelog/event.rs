use crate::protos::gamelog as log;
use crate::protos::{bed, bp, cai, grav, halloween, herd, timv};

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
    TimvDetectiveBody(timv::DetectiveBodyEvent),
    TimvPsychicReport(timv::SubrolePsychicReportEvent),
    TimvSharedPurchase(timv::SharedPointsPurchaseEvent),

    // BP events
    BpDeath(bp::DeathEvent),
    BpRound(bp::RoundEvent),
    BpWinners(bp::WinnersEvent),
    BpPowerup(bp::PowerUpEvent),

    // GRAV events
    GravStageCompletion(grav::StageCompletionEvent),
    GravGameFinish(grav::GameFinishEvent),
    GravHardcoreFail(grav::HardcoreModeFailEvent),

    // Herd events
    HerdDeath(herd::DeathEvent),
    HerdElimination(herd::EliminationEvent),

    // BED events
    BedBedDestruction(bed::BedDestructionEvent),

    // Halloween events
    HalloweenDeath(halloween::DeathEvent),
}
