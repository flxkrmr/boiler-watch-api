use crate::temperature_recorder::{RecorderConfig, Temperature, TemperaturesByTime};
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
    Write(rusqlite::Error),
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

        connection
            .execute(
                "create table if not exists temperatures (
                name string not null,
                value real not null,
                date integer not null )",
                (),
            )
            .map_err(DatabaseInitError::CreateDatabases)?;

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

    pub fn save_recorder_config(
        &self,
        config: RecorderConfig,
    ) -> Result<RecorderConfig, DatabaseAccessError> {
        self.connection
            .execute("delete from recorder_config", ())
            .map_err(DatabaseAccessError::Write)?;

        self.connection
            .execute(
                "insert into recorder_config (interval_seconds, delete_older_seconds) values (?1, ?2)",
                (&config.interval_seconds, &config.delete_older_seconds),
            )
            .map_err(DatabaseAccessError::Write)?;

        Ok(config)
    }

    pub fn load_temperatures_since(
        &self,
        since: i64,
    ) -> Result<Vec<TemperaturesByTime>, DatabaseAccessError> {
        let dates = self.load_dates_distinct_since(since)?;

        log::debug!("Dates: {:?}", dates);

        return dates
            .iter()
            .map(|date| {
                let mut statement = self
                    .connection
                    .prepare(
                        "select name, value
                        from temperatures where date is ?1",
                    )
                    .map_err(DatabaseAccessError::Read)?;

                let temperatures = statement
                    .query_map([date], |row| {
                        let name = row.get(0)?;
                        let value = row.get(1)?;
                        return Ok(Temperature::new(name, value));
                    })
                    .map_err(DatabaseAccessError::Read)?
                    .collect::<Result<Vec<Temperature>, _>>()
                    .map_err(DatabaseAccessError::Read)?;

                let temperatures_by_time = TemperaturesByTime::new(date.to_string(), temperatures);

                Ok(temperatures_by_time)
            })
            .collect();
    }

    pub fn load_last_temperature(&self) -> Result<Option<TemperaturesByTime>, DatabaseAccessError> {
        let date_max_opt = self.load_youngest_date_of_temperatures()?;

        if date_max_opt.is_none() {
            return Ok(None);
        }
        // TODO ugly...
        let date_max = date_max_opt.unwrap();

        log::debug!("date_max: {}", date_max);

        let mut statement = self
            .connection
            .prepare(
                "select name, value
                from temperatures where date = ?1",
            )
            .map_err(DatabaseAccessError::Read)?;

        let temperatures = statement
            .query_map([date_max], |row| {
                let name = row.get(0)?;
                let value = row.get(1)?;
                return Ok(Temperature::new(name, value));
            })
            .map_err(DatabaseAccessError::Read)?
            .collect::<Result<Vec<Temperature>, _>>()
            .map_err(DatabaseAccessError::Read)?;

        return Ok(Some(TemperaturesByTime::new(
            date_max.to_string(),
            temperatures,
        )));
    }

    fn load_dates_distinct_since(&self, since: i64) -> Result<Vec<u64>, DatabaseAccessError> {
        let mut statement = self
            .connection
            .prepare(
                "select distinct date from temperatures
                    where date >= ?1",
            )
            .map_err(DatabaseAccessError::Read)?;

        let dates = statement
            .query_map([since], |row| {
                let date: u64 = row.get(0)?;
                Ok(date)
            })
            .map_err(DatabaseAccessError::Read)?
            .collect::<Result<Vec<u64>, _>>()
            .map_err(DatabaseAccessError::Read)?;

        Ok(dates)
    }

    fn load_youngest_date_of_temperatures(&self) -> Result<Option<u64>, DatabaseAccessError> {
        let mut statement = self
            .connection
            .prepare("select max(date) from temperatures")
            .map_err(DatabaseAccessError::Read)?;

        let mut date_iter = statement
            .query_map([], |row| {
                let max_date: u64 = row.get(0)?;
                Ok(max_date)
            })
            .map_err(DatabaseAccessError::Read)?;

        if let Some(max_date) = date_iter.next() {
            let max_date = max_date.map_err(DatabaseAccessError::Read)?;
            return Ok(Some(max_date));
        } else {
            return Ok(None);
        }
    }
}
