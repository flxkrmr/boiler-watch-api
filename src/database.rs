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
    Delete(rusqlite::Error),
    NoConfigFound,
}

impl Database {
    pub fn new() -> Result<Self, DatabaseInitError> {
        // TODO get path from arguments
        let connection = Connection::open("boiler-watch.db").map_err(DatabaseInitError::Open)?;

        connection
            .execute(
                "create table if not exists recorder_config (
                 interval_seconds integer not null,
                 keep_days integer not null )",
                (),
            )
            .map_err(DatabaseInitError::CreateDatabases)?;

        connection
            .execute(
                "insert into recorder_config (interval_seconds, keep_days)
                select 15, 30
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
                "select interval_seconds, keep_days 
                from recorder_config",
            )
            .map_err(DatabaseAccessError::Read)?;

        let mut config_iter = statement
            .query_map([], |row| {
                let interval_seconds = row.get(0)?;
                let keep_days = row.get(1)?;
                return Ok(RecorderConfig::new(interval_seconds, keep_days));
            })
            .map_err(DatabaseAccessError::Read)?;

        if let Some(config) = config_iter.next() {
            config.map_err(DatabaseAccessError::Read)
        } else {
            Err(DatabaseAccessError::NoConfigFound)
        }
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
                "insert into recorder_config (interval_seconds, keep_days) values (?1, ?2)",
                (&config.interval_seconds, &config.keep_days),
            )
            .map_err(DatabaseAccessError::Write)?;

        Ok(config)
    }

    pub fn load_temperatures_since(
        &self,
        since: u64,
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

                let temperatures_by_time = TemperaturesByTime::new(date.to_owned(), temperatures);

                Ok(temperatures_by_time)
            })
            .collect();
    }

    pub fn load_last_temperature(&self) -> Result<Option<TemperaturesByTime>, DatabaseAccessError> {
        let date_max_opt = self.load_youngest_date_of_temperatures()?;
        let date_max: u64;

        if let Some(d) = date_max_opt {
            date_max = d;
        } else {
            return Ok(None);
        }

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

        return Ok(Some(TemperaturesByTime::new(date_max, temperatures)));
    }

    fn load_dates_distinct_since(&self, since: u64) -> Result<Vec<u64>, DatabaseAccessError> {
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
                Ok(max_date as u64)
            })
            .map_err(DatabaseAccessError::Read)?;

        if let Some(max_date) = date_iter.next() {
            let max_date = max_date.map_err(DatabaseAccessError::Read)?;
            return Ok(Some(max_date));
        } else {
            return Ok(None);
        }
    }

    pub fn save_temperatures(
        &self,
        temperatures_by_time: TemperaturesByTime,
    ) -> Result<(), DatabaseAccessError> {
        let saving_result: Result<Vec<usize>, DatabaseAccessError> = temperatures_by_time
            .temperatures()
            .iter()
            .map(|temperature| {
                self.connection
                    .execute(
                        "insert into temperatures (date, name, value) values (?1, ?2, ?3)",
                        (
                            temperatures_by_time.date(),
                            temperature.name(),
                            temperature.value_rounded_as_string(),
                        ),
                    )
                    .map_err(DatabaseAccessError::Write)
            })
            .collect();

        match saving_result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn delete_old_temperatures(&self) -> Result<usize, DatabaseAccessError> {
        let config = self.load_recorder_config()?;
        let keep_days = config.keep_days;

        self.connection
            .execute(
                "delete from temperatures where date < (strftime('%s', 'now') - ?1 * 86400) * 1000",
                [keep_days],
            )
            .map_err(DatabaseAccessError::Delete)
    }
}
