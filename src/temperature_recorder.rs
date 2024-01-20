use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RecorderConfig {
    pub interval_seconds: u64,
    pub delete_older_seconds: u64,
}

impl RecorderConfig {
    pub fn new(interval_seconds: u64, delete_older_seconds: u64) -> Self {
        return RecorderConfig {
            interval_seconds,
            delete_older_seconds,
        };
    }
}

#[derive(Serialize)]
struct Temperature {
    name: String,
    value: f32,
}

#[derive(Serialize)]
pub struct TemperaturesByTime {
    date: String,
    temperatures: Vec<Temperature>,
}

pub struct TemperatureRecorder {}

impl TemperatureRecorder {
    pub fn last() -> TemperaturesByTime {
        // TODO
        let temperature1 = Temperature {
            name: String::from("hello"),
            value: 33.0,
        };

        let temperature2 = Temperature {
            name: String::from("you"),
            value: 36.30,
        };

        let temperatures = vec![temperature1, temperature2];
        let date = Utc::now().timestamp().to_string();
        TemperaturesByTime { date, temperatures }
    }

    pub fn since(_date: DateTime<Utc>) -> Vec<TemperaturesByTime> {
        // TODO
        let temp1 = TemperatureRecorder::last();
        let temp2 = TemperatureRecorder::last();
        let temp3 = TemperatureRecorder::last();

        vec![temp1, temp2, temp3]
    }
}
