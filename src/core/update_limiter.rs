use std::time::{Instant,Duration};
use std::thread::{sleep,yield_now};

#[derive(Clone)]
pub enum LimitSetting {
    Unlimited,
    Yield(u32),
    Sleep(u32),
    SleepAndYield(u32)
}

impl Default for LimitSetting {
    fn default() -> Self {
        LimitSetting::Sleep(60)
    }
}

pub struct UpdateLimiter {
    frame_duration:Duration,
    setting:LimitSetting,
    last_call:Instant
}

impl Default for UpdateLimiter {
    fn default() -> Self {
        UpdateLimiter::new(LimitSetting::default())
    }
}

impl UpdateLimiter {

    pub fn new(setting:LimitSetting) -> Self {
        let mut ret = UpdateLimiter {
            frame_duration:Duration::from_secs(0),
            setting:Default::default(),
            last_call:Instant::now()
        };
        ret.set(setting);
        ret
    }

    pub fn start(&mut self) {
        self.last_call = Instant::now();
    }

    pub fn set(&mut self,setting:LimitSetting) {
        self.setting = setting;
        match self.setting {
            LimitSetting::Yield(fps) => self.frame_duration = Duration::from_secs(1) / fps,
            LimitSetting::Sleep(fps) => self.frame_duration = Duration::from_secs(1) / fps,
            LimitSetting::SleepAndYield(fps) => self.frame_duration = Duration::from_secs(1) / fps,
            LimitSetting::Unlimited => ()
        }
    }

    pub fn wait(&mut self) {
        match self.setting {
            LimitSetting::Unlimited => yield_now(),
            LimitSetting::Yield(_)  => self.do_yield(),
            LimitSetting::Sleep(_)  => self.do_sleep(),
            LimitSetting::SleepAndYield(_) => {
                self.do_sleep();
                self.do_yield();
            }
        }
        self.last_call = Instant::now();
    }

    fn do_yield(&self) {
        while Instant::now() - self.last_call < self.frame_duration {
            yield_now()
        }
    }

    fn do_sleep(&self) {
        loop {
            let elapsed = Instant::now() - self.last_call;
            if elapsed >= self.frame_duration {
                break
            } else {
                sleep(self.frame_duration - elapsed)
            }
        }
    }
}