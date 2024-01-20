use crate::temperature_recorder::RecorderConfig;
use rusqlite::Connection;

pub struct Database {
    connection: Connection,
}

#[derive(Debug)]
pub enum DatabaseInitError {
    Open(rusqlite::Error),
    CreateDatabases(rusqlite::Error),
    InsertDefaultState(rusqlite::Error),
}

#[derive(Debug)]
pub enum DatabaseAccessError {
    Read(rusqlite::Error),
}

impl Database {
    pub fn new() -> Result<Self, DatabaseInitError> {
        let connection = Connection::open("boiler-watch.db").map_err(DatabaseInitError::Open)?;

        connection
            .execute(
                "create table if not exists recorder_config (
                 interval_seconds integer not null,
                 delete_older_seconds integer not null )",
                (),
            )
            .map_err(DatabaseInitError::CreateDatabases)?;

        connection
            .execute(
                "insert into recorder_config (interval_seconds, delete_older_seconds)
                select 15, 60
                where not exists (select * from recorder_config)",
                (),
            )
            .map_err(DatabaseInitError::InsertDefaultState)?;

        Ok(Database { connection })
    }

    pub fn load_recorder_config(&self) -> Result<RecorderConfig, DatabaseAccessError> {
        let mut statement = self
            .connection
            .prepare(
                "select interval_seconds, delete_older_seconds 
                from recorder_config",
            )
            .map_err(DatabaseAccessError::Read)?;

        let mut config_iter = statement
            .query_map([], |row| {
                let interval_seconds = row.get(0)?;
                let delete_older_seconds = row.get(1)?;
                return Ok(RecorderConfig::new(interval_seconds, delete_older_seconds));
            })
            .map_err(DatabaseAccessError::Read)?;

        // TODO fix unwrap here
        let config = config_iter.next().unwrap();

        return config.map_err(DatabaseAccessError::Read);
    }
}
