use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RecorderConfig {
    pub interval_seconds: u64,
    pub delete_older_seconds: u64,
}

impl RecorderConfig {
    pub fn new(interval_seconds: u64, delete_older_seconds: u64) -> Self {
        return Self {
            interval_seconds,
            delete_older_seconds,
        };
    }
}

#[derive(Serialize)]
pub struct Temperature {
    name: String,
    value: f32,
}

impl Temperature {
    pub fn new(name: String, value: f32) -> Self {
        Self { name, value }
    }
}

#[derive(Serialize)]
pub struct TemperaturesByTime {
    date: String,
    temperatures: Vec<Temperature>,
}

impl TemperaturesByTime {
    pub fn new(date: String, temperatures: Vec<Temperature>) -> Self {
        return Self { date, temperatures };
    }
}

pub struct TemperatureRecorder {}
