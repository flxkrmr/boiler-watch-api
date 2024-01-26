use crate::temperature_recorder::RecorderConfig;

use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use std::time::Duration;

pub struct RecorderScheduler {
    thread: Option<ScheduleHandle>,
}

impl RecorderScheduler {
    pub fn new() -> Self {
        return Self { thread: None };
    }

    pub fn start(&mut self, config: &RecorderConfig) {
        let interval = config.interval_seconds;

        let mut scheduler = Scheduler::new();
        scheduler
            .every(interval.seconds())
            .run(|| println!("Say Hi!"));

        let thread = scheduler.watch_thread(Duration::from_millis(100));

        self.thread = Some(thread);
    }

    pub fn stop(&mut self) {
        self.thread = None;
    }
}
