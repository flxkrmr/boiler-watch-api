pub mod temperature_recorder;

use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;

use crate::temperature_recorder::{State, TemperatureRecorder, TemperaturesByTime};

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
fn state() -> Json<State> {
    let state = State::load();
    Json::from(state)
}

#[post("/state", data = "<state>")]
fn update_state(state: Json<State>) -> Json<State> {
    let state = State::save(state.into_inner());
    Json::from(state)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![last_temperatures, temperatures_since, state, update_state],
    )
}
