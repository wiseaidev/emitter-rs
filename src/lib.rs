#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod event_emitter;
pub mod event_emitter_file;
pub use event_emitter::EventEmitter;
