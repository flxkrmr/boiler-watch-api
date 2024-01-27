use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RecorderConfig {
    pub interval_seconds: u32,
    pub keep: u64,
}

impl RecorderConfig {
    pub fn new(interval_seconds: u32, keep: u64) -> Self {
        return Self {
            interval_seconds,
            keep,
        };
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
    date: String,
    temperatures: Vec<Temperature>,
}

impl TemperaturesByTime {
    pub fn new(date: String, temperatures: Vec<Temperature>) -> Self {
        return Self { date, temperatures };
    }

    pub fn date(&self) -> String {
        self.date.to_owned()
    }

    pub fn temperatures(&self) -> Vec<Temperature> {
        self.temperatures.clone()
    }
}
