pub mod database;
pub mod recorder_scheduler;
pub mod temperature_recorder;

use rocket::serde::json::Json;
use rocket::State;
use std::sync::{Arc, Mutex};

use crate::database::{Database, DatabaseAccessError, DatabaseInitError};
use crate::recorder_scheduler::RecorderScheduler;
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
    start_time: i64,
    state: &State<AppState>,
) -> Result<Json<Vec<TemperaturesByTime>>, ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::warn!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let temperatures = db.load_temperatures_since(start_time).map_err(|err| {
        log::warn!("Error accessing database: {:?}", err);
        ResponseError::Internal(String::from("Error accessing database"))
    })?;

    return Ok(Json::from(temperatures));
}

#[get("/config")]
fn get_config(state: &State<AppState>) -> Result<Json<RecorderConfig>, ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::warn!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let recorder_config = db.load_recorder_config().map_err(|err| {
        log::warn!("Error loading recorder config: {:?}", err);
        ResponseError::Internal(String::from("Error loading recorder config"))
    })?;

    Ok(Json::from(recorder_config))
}

#[post("/config", data = "<recorder_config>")]
fn save_config(
    recorder_config: Json<RecorderConfig>,
    state: &State<AppState>,
) -> Result<(), ResponseError> {
    let db = state.db.lock().map_err(|err| {
        log::warn!("Error retreiving database from state: {}", err);
        ResponseError::Internal(String::from("Error retreiving database from state"))
    })?;

    let recorder_config = &db
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
    current_scheduler.start(recorder_config);

    Ok(())
}

#[derive(Debug)]
enum StartupError {
    Api(rocket::Error),
    DatabaseInit(DatabaseInitError),
    DatabaseAccess(DatabaseAccessError),
}

struct AppState {
    db: Arc<Mutex<Database>>,
    scheduler: Arc<Mutex<RecorderScheduler>>,
}

#[rocket::main]
async fn main() -> Result<(), StartupError> {
    let db = Database::new().map_err(StartupError::DatabaseInit)?;

    let recorder_config = &db
        .load_recorder_config()
        .map_err(StartupError::DatabaseAccess)?;

    let db = Arc::new(Mutex::new(db));

    let mut scheduler = RecorderScheduler::new();
    scheduler.start(recorder_config);
    let scheduler = Arc::new(Mutex::new(scheduler));

    rocket::build()
        .manage(AppState { db, scheduler })
        .mount(
            "/",
            routes![
                get_last_temperatures,
                get_temperatures_since,
                get_config,
                save_config
            ],
        )
        .launch()
        .await
        .map_err(StartupError::Api)?;

    Ok(())
}
