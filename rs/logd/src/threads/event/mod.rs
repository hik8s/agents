mod error;
mod event;

pub use error::EventThreadError;
pub use event::process_inotify_events;
