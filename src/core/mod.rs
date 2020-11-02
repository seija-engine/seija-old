use shrev::{EventChannel,ReaderId,EventIterator};
mod update_limiter;
mod timing;
mod game;
mod module_bundle;

pub use update_limiter::{UpdateLimiter,LimitSetting};
pub use timing::{Stopwatch,Time};
pub use module_bundle::{IModuleBundle};
pub use game::{IGame};

#[derive(Copy,Clone)]
pub enum AppControlFlow {
    None,
    Quit,
    Pause
}

pub struct EventHandler<T:'static> {
   pub channel:EventChannel<T>,
   pub reader:ReaderId<T>
}

impl<T> EventHandler<T> where T:Sync + Send {
    pub fn new() -> Self {
        let mut channel = EventChannel::new();
        let reader = channel.register_reader();
        EventHandler {
            channel: channel,
            reader: reader
        }
    }

    pub fn read(&mut self) -> EventIterator<T> {
        self.channel.read(&mut self.reader)
    }
}