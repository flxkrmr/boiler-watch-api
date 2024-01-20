pub mod database;
pub mod temperature_recorder;

use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::{Arc, Mutex};

use crate::database::{Database, DatabaseInitError};
use crate::temperature_recorder::{RecorderConfig, TemperatureRecorder, TemperaturesByTime};

#[macro_use]
extern crate rocket;

#[get("/temperatures/last")]
fn last_temperatures() -> Json<TemperaturesByTime> {
    let temperatures = TemperatureRecorder::last();
    Json::from(temperatures)
}

#[get("/temperatures/since/<start_time>")]
fn temperatures_since(
    start_time: i64,
) -> Result<Json<Vec<TemperaturesByTime>>, BadRequest<String>> {
    let date = parse_start_time(start_time)?;
    let temperatures = TemperatureRecorder::since(date);

    return Ok(Json::from(temperatures));
}

fn parse_start_time(start_time: i64) -> Result<DateTime<Utc>, BadRequest<String>> {
    let date = NaiveDateTime::from_timestamp_millis(start_time);

    if date.is_none() {
        return Err(BadRequest(String::from("Invalid start time given")));
    }

    let date = date.unwrap();
    let offset = Utc::now().offset().to_owned();

    return Ok(<DateTime<Utc>>::from_naive_utc_and_offset(date, offset));
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
