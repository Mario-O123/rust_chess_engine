use crate::position::Position;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct EngineState {
    root_position: Position,
    search_running: bool,
    stop_flag: Arc<AtomicBool>,
}
