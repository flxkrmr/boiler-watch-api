pub mod database;
pub mod temperature_recorder;

use rocket::serde::json::Json;
use rocket::State;
use std::sync::{Arc, Mutex};

use crate::database::{Database, DatabaseInitError};
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
fn last_temperatures(state: &State<AppState>) -> Result<Json<TemperaturesByTime>, ResponseError> {
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
fn temperatures_since(
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

#[get("/state")]
fn state(state: &State<AppState>) -> Json<RecorderConfig> {
    let db = state.db.lock().unwrap();
    let recorder_config = db.load_recorder_config().unwrap();
    Json::from(recorder_config)
}

#[post("/state", data = "<recorder_config>")]
fn update_state(
    recorder_config: Json<RecorderConfig>,
    state: &State<AppState>,
) -> Json<RecorderConfig> {
    let db = state.db.lock().unwrap();
    let recorder_config = db
        .save_recorder_config(recorder_config.into_inner())
        .unwrap();

    Json::from(recorder_config)
}

#[derive(Debug)]
enum StartupError {
    Api(rocket::Error),
    Database(DatabaseInitError),
}

struct AppState {
    db: Arc<Mutex<Database>>,
}

#[rocket::main]
async fn main() -> Result<(), StartupError> {
    let db = Database::new().map_err(StartupError::Database)?;
    let db = Arc::new(Mutex::new(db));

    rocket::build()
        .manage(AppState { db })
        .mount(
            "/",
            routes![last_temperatures, temperatures_since, state, update_state],
        )
        .launch()
        .await
        .map_err(StartupError::Api)?;

    Ok(())
}
