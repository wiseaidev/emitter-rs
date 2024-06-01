use crate::EventEmitter;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
}
