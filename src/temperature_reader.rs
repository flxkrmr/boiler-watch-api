use crate::temperature_recorder::Temperature;

use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::num::ParseIntError;

pub struct TemperatureReader;

#[derive(Debug)]
pub enum TemperatureReaderError {
    ConfigRead(std::io::Error),
    ConfigParse(toml::de::Error),
    SensorRead(std::io::Error, Sensor),
    SensorParse(ParseIntError, Sensor, String),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SensorConfig {
    sensors: Vec<Sensor>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Sensor {
    name: String,
    path: String,
}

impl TemperatureReader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_config(config_file_path: &str) -> Result<SensorConfig, TemperatureReaderError> {
        let sensor_file =
            read_to_string(config_file_path).map_err(TemperatureReaderError::ConfigRead)?;
        let sensor_config: SensorConfig =
            toml::from_str(&sensor_file).map_err(TemperatureReaderError::ConfigParse)?;

        Ok(sensor_config)
    }

    pub fn read(&self) -> Result<Vec<Temperature>, TemperatureReaderError> {
        // TODO use app arguments
        let sensor_config = Self::read_config("Sensor.toml")?;

        let mut errors = vec![];
        let temperatures: Vec<Temperature> = sensor_config
            .sensors
            .iter()
            .filter_map(|sensor| {
                let temperature = Self::read_sensor(&sensor);
                temperature
                    .map_err(|e| errors.push(e))
                    .ok()
                    .map(|t| Temperature::new(sensor.name.clone(), t))
            })
            .collect();

        if !errors.is_empty() {
            log::error!("Error reading sensors {:?}", errors);
        }

        return Ok(temperatures);
    }

    fn read_sensor(sensor: &Sensor) -> Result<f32, TemperatureReaderError> {
        let sensor_file_content = read_to_string(&sensor.path)
            .map_err(|e| TemperatureReaderError::SensorRead(e, sensor.to_owned()))?;

        let sensor_value = sensor_file_content.trim_end().parse::<i32>().map_err(|e| {
            TemperatureReaderError::SensorParse(e, sensor.to_owned(), sensor_file_content)
        })?;

        let temperature = sensor_value as f32 / 1000.0;

        Ok(temperature)
    }
}
