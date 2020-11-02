use std::time::{Duration,Instant};
#[derive(Clone,Eq,PartialEq)]
pub enum Stopwatch {
    Waiting,
    Started(Duration,Instant),
    Ended(Duration)
}

impl Default for Stopwatch {
    fn default() -> Stopwatch {
        Stopwatch::Waiting
    }
}

impl Stopwatch {
    pub fn new() -> Stopwatch {
        Default::default()
    }

    pub fn elapsed(&self) -> Duration {
        match *self {
            Stopwatch::Waiting => Duration::new(0, 0),
            Stopwatch::Started(dur,start) => dur + start.elapsed(),
            Stopwatch::Ended(dur) => dur
        }
    }

    pub fn restart(&mut self) {
        *self = Stopwatch::Started(Duration::new(0, 0),Instant::now());
    }

    pub fn start(&mut self) {
        match *self {
            Stopwatch::Waiting => self.restart(),
            Stopwatch::Ended(dur) => {
                *self = Stopwatch::Started(dur, Instant::now());
            },
            _ => ()
        }
    }

    pub fn stop(&mut self) {
        if let Stopwatch::Started(dur,start) = *self {
            *self = Stopwatch::Ended(dur + start.elapsed());
        }
    }

    pub fn reset(&mut self) {
        *self = Stopwatch::Waiting;
    }

}


pub struct Time {
    frame_number: u64,
    delta_real_time: Duration,
    delta_seconds: f32,
    time_scale: f32,
    delta_time: Duration,
    delta_real_seconds: f32,
    absolute_real_time: Duration,
    absolute_time: Duration,
}

impl Time {

    pub fn inc_frame_number(&mut self) {
        self.frame_number += 1;
    }

    pub fn frame_number(&self) -> u64 {
        return self.frame_number
    }
    
    pub fn set_time_scale(&mut self, multiplier: f32) {
        use std::f32::INFINITY;
        assert!(multiplier >= 0.0);
        assert!(multiplier != INFINITY);
        self.time_scale = multiplier;
    }

    pub fn time_scale(&self) -> f32 {
        return self.time_scale
    }

    pub fn set_delta_time(&mut self,time: Duration) {
        self.delta_seconds = duration_to_secs(time) * self.time_scale;
        self.delta_time = secs_to_duration(duration_to_secs(time) * self.time_scale);
        self.delta_real_seconds = duration_to_secs(time);
        self.delta_real_time = time;

        self.absolute_time += self.delta_time;
        self.absolute_real_time += self.delta_real_time;
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn delta_real_seconds(&self) -> f32 {
        self.delta_real_seconds
    }

    pub fn absolute_time(&self) -> Duration {
        self.absolute_time
    }

    pub fn absolute_time_seconds(&self) -> f64 {
        duration_to_secs_f64(self.absolute_time)
    }

    pub fn absolute_real_time(&self) -> Duration {
        self.absolute_real_time
    }

    pub fn absolute_real_time_seconds(&self) -> f64 {
        duration_to_secs_f64(self.absolute_real_time)
    }
}

impl Default for Time {
    fn default() -> Time {
        Time { 
            frame_number: 0,
            delta_real_time:Duration::from_secs(0),
            time_scale: 1f32,
            delta_seconds: 0f32,
            delta_time: Duration::from_secs(0),
            delta_real_seconds: 0f32,
            absolute_real_time: Duration::default(),
            absolute_time: Duration::default(),
        }
    }
}



/// Converts a Duration to the time in seconds.
pub fn duration_to_secs(duration: Duration) -> f32 {
    duration.as_secs() as f32 + (duration.subsec_nanos() as f32 / 1.0e9)
}

/// Converts a Duration to the time in seconds in an f64.
pub fn duration_to_secs_f64(duration: Duration) -> f64 {
    duration.as_secs() as f64 + (f64::from(duration.subsec_nanos()) / 1.0e9)
}

/// Converts a time in seconds to a duration
pub fn secs_to_duration(secs: f32) -> Duration {
    Duration::new(secs as u64, ((secs % 1.0) * 1.0e9) as u32)
}

/*
/// Converts a Duration to nanoseconds
pub fn duration_to_nanos(duration: Duration) -> u64 {
    (duration.as_secs() * 1_000_000_000) + u64::from(duration.subsec_nanos())
}

/// Converts nanoseconds to a Duration
pub fn nanos_to_duration(nanos: u64) -> Duration {
    Duration::new(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32)
}*/
