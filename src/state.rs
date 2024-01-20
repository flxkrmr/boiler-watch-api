use serde::Serialize;

#[derive(Serialize)]
pub struct State {
    interval_seconds: u32,
}
