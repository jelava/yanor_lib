/// Tick-based analogue to Bevy's Stopwatch struct, although it does not aim to exactly match the
/// interface or feature set of Bevy's stopwatch implementation.
#[derive(Default)]
pub struct TickStopwatch {
    elapsed_ticks: u32,
}

impl TickStopwatch {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn elapsed_ticks(&self) -> u32 {
        self.elapsed_ticks
    }

    pub fn tick(&mut self, delta: u32) -> &Self {
        self.elapsed_ticks += delta;
        self
    }

    pub fn reset(&mut self) {
        self.elapsed_ticks = 0;
    }
}

/// Tick-based analogue to Bevy's Timer struct, although it does not aim to exactly match the
/// interface or feature set of Bevy's timers.
pub struct TickTimer {
    duration: u32,
    stopwatch: TickStopwatch,
}

impl TickTimer {
    pub fn new(duration: u32) -> Self {
        Self {
            duration,
            stopwatch: TickStopwatch::new(),
        }
    }

    pub fn elapsed_ticks(&self) -> u32 {
        self.stopwatch.elapsed_ticks()
    }

    pub fn tick(&mut self, delta: u32) -> &Self {
        self.stopwatch.tick(delta);
        self
    }

    pub fn finished(&self) -> bool {
        self.stopwatch.elapsed_ticks >= self.duration
    }

    pub fn fraction(&self) -> f32 {
        let duration = self.duration as f32;
        let elapsed = f32::min(self.stopwatch.elapsed_ticks as f32, duration);

        elapsed / duration
    }

    pub fn reset(&mut self) {
        self.stopwatch.reset();
    }

    pub fn set_duration(&mut self, duration: u32) {
        self.duration = duration;
    }
}
