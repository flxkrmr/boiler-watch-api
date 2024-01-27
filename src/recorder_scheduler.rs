use crate::database::{Database, DatabaseInitError};
use crate::temperature_reader::TemperatureReader;
use crate::temperature_recorder::{RecorderConfig, TemperaturesByTime};

use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

pub struct RecorderScheduler {
    thread: Option<ScheduleHandle>,
}

#[derive(Debug)]
pub enum RecorderSchedulerError {
    Database(DatabaseInitError),
    Date(SystemTimeError),
}

impl RecorderScheduler {
    pub fn new() -> Self {
        return Self { thread: None };
    }

    pub fn start(&mut self, config: &RecorderConfig) -> Result<(), RecorderSchedulerError> {
        let interval = config.interval_seconds;
        let db = Database::new().map_err(RecorderSchedulerError::Database)?;

        let mut scheduler = Scheduler::new();
        scheduler.every(interval.seconds()).run(move || {
            let reader = TemperatureReader::new();
            let date;
            let date_res = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis());

            // TODO make this more pretty
            if date_res.is_err() {
                log::error!("Error reading time");
                return;
            } else {
                date = date_res.unwrap();
            }

            match reader.read() {
                Ok(temperatures) => {
                    let temperatures_by_time =
                        TemperaturesByTime::new(date.to_string(), temperatures);

                    log::debug!("Successfully read sensors: {:?}", temperatures_by_time);

                    if let Err(error) = db.save_temperatures(temperatures_by_time) {
                        log::error!("Error saving temperatures to database {:?}", error);
                    } else {
                        log::debug!("Saved temperatures to database");
                    }
                }
                Err(error) => log::error!("Error reading sensors {:?}", error),
            }
        });

        let thread = scheduler.watch_thread(Duration::from_millis(100));

        self.thread = Some(thread);

        Ok(())
    }

    pub fn stop(&mut self) {
        self.thread = None;
    }
}
