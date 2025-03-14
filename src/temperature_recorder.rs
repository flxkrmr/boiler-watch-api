use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RecorderConfig {
    pub interval_seconds: u32,
    pub keep_days: u64,
}

impl RecorderConfig {
    pub fn new(interval_seconds: u32, keep_days: u64) -> Self {
        Self {
            interval_seconds,
            keep_days,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Temperature {
    name: String,
    value: f32,
}

impl Temperature {
    pub fn new(name: String, value: f32) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn value_rounded_as_string(&self) -> String {
        format!("{:.2}", self.value)
    }
}

#[derive(Serialize, Debug)]
pub struct TemperaturesByTime {
    date: u64,
    temperatures: Vec<Temperature>,
}

impl TemperaturesByTime {
    pub fn new(date: u64, temperatures: Vec<Temperature>) -> Self {
        Self { date, temperatures }
    }

    pub fn date(&self) -> u64 {
        self.date
    }

    pub fn temperatures(&self) -> Vec<Temperature> {
        self.temperatures.clone()
    }
}
