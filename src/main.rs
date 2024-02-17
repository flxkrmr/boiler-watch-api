pub mod database;
pub mod recorder_scheduler;
pub mod temperature_reader;
pub mod temperature_recorder;

use filesize::PathExt;
use rocket::serde::json::Json;
use rocket::State;
use rocket_cors::CorsOptions;
use serde::Serialize;
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::database::{Database, DatabaseAccessError, DatabaseInitError};
use crate::recorder_scheduler::{RecorderScheduler, RecorderSchedulerError};
use crate::temperature_reader::{SensorConfig, TemperatureReader};
use crate::temperature_recorder::{RecorderConfig, TemperaturesByTime};

#[macro_use]
extern crate rocket;

#[derive(Responder)]
enum ResponseError {
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 500, content_type = "json")]
    Internal(String),
}

// TODO endpoints:
// clear all temperatures
// logs??

#[get("/temperatures/last")]
fn get_last_temperatures(
    state: &State<AppState>,
) -> Result<Json<TemperaturesByTime>, ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::warn!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let last_temperatures = db.load_last_temperature().map_err(|err| {
        log::warn!("Error accessing database: {:?}", err);
        ResponseError::Internal(String::from("Error accessing database"))
    })?;

    return match last_temperatures {
        Some(t) => Ok(Json::from(t)),
        None => Err(ResponseError::NotFound(String::from(
            "No temperatures found",
        ))),
    };
}

#[get("/temperatures/since/<start_time>")]
fn get_temperatures_since(
    start_time: u64,
    state: &State<AppState>,
) -> Result<Json<Vec<TemperaturesByTime>>, ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::error!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let temperatures = db.load_temperatures_since(start_time).map_err(|err| {
        log::error!("Error accessing database: {:?}", err);
        ResponseError::Internal(String::from("Error accessing database"))
    })?;

    return Ok(Json::from(temperatures));
}

#[derive(Serialize)]
struct AppHealth {
    sensor_config: SensorConfig,
    database_size_bytes: u64,
}

#[get("/health")]
fn get_app_health() -> Result<Json<AppHealth>, ResponseError> {
    let sensor_config = TemperatureReader::read_config("Sensor.toml").map_err(|error| {
        log::error!("Error reading sensor configuration file: {:?}", error);
        ResponseError::Internal(String::from("Error reading sensor configuration file"))
    })?;

    let path = Path::new("boiler-watch.db");
    let database_size_bytes = path.size_on_disk().map_err(|error| {
        log::error!("Error getting database file size: {:?}", error);
        ResponseError::Internal(String::from("Error getting database file size"))
    })?;

    let health = AppHealth {
        sensor_config,
        database_size_bytes,
    };

    Ok(Json::from(health))
}

#[get("/config")]
fn get_config(state: &State<AppState>) -> Result<Json<RecorderConfig>, ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::error!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let recorder_config = db.load_recorder_config().map_err(|err| {
        log::error!("Error loading recorder config: {:?}", err);
        ResponseError::Internal(String::from("Error loading recorder config"))
    })?;

    Ok(Json::from(recorder_config))
}

#[post("/config", data = "<recorder_config>")]
fn save_config(
    recorder_config: Json<RecorderConfig>,
    state: &State<AppState>,
) -> Result<Json<RecorderConfig>, ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::warn!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let recorder_config = db
        .save_recorder_config(recorder_config.into_inner())
        .map_err(|err| {
            log::warn!("Error saving new recorder config: {:?}", err);
            ResponseError::Internal(String::from("Error saving new recorder config"))
        })?;

    let mut current_scheduler = state.scheduler.lock().map_err(|err| {
        log::warn!("Error retreiving scheduler from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving scheduler from state"))
    })?;

    current_scheduler.stop();
    // TODO catch error
    current_scheduler.start(&recorder_config).unwrap();

    Ok(Json::from(recorder_config))
}

#[derive(Debug)]
enum StartupError {
    Api(rocket::Error),
    DatabaseInit(DatabaseInitError),
    DatabaseAccess(DatabaseAccessError),
    Scheduler(RecorderSchedulerError),
}

struct AppState {
    db: Arc<Mutex<Database>>,
    scheduler: Arc<Mutex<RecorderScheduler>>,
}

#[rocket::main]
async fn main() -> Result<(), StartupError> {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let db = Database::new().map_err(StartupError::DatabaseInit)?;

    let recorder_config = &db
        .load_recorder_config()
        .map_err(StartupError::DatabaseAccess)?;

    let db = Arc::new(Mutex::new(db));

    let mut scheduler = RecorderScheduler::new();

    scheduler
        .start(recorder_config)
        .map_err(StartupError::Scheduler)?;

    let scheduler = Arc::new(Mutex::new(scheduler));

    let cors_options = CorsOptions::default();
    let cors = cors_options.to_cors().unwrap();

    rocket::build()
        .attach(cors)
        .manage(AppState { db, scheduler })
        .mount(
            "/",
            routes![
                get_last_temperatures,
                get_temperatures_since,
                get_config,
                save_config,
                get_app_health
            ],
        )
        .launch()
        .await
        .map_err(StartupError::Api)?;

    Ok(())
}
