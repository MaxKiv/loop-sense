use std::sync::Arc;

use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use influxdb::{Client, InfluxDbWriteable as _, WriteQuery};
use loop_sense::controller::backend::mockloop_hardware::SensorData;
use loop_sense::database::query::GET_LATEST_MEASUREMENT_ID;
use loop_sense::database::record::SensorDataRecord;
use loop_sense::database::secrets::{DB_ACCESS_TOKEN, DB_NAME, DB_URI, MEASUREMENT_ID_TABLE};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, Duration, Instant};
use tracing::{debug, error, info, warn};

const DB_LOOP_PERIOD: Duration = Duration::from_millis(10);
const QUERY_BATCH_LEN: usize = 100;
const DEFAULT_MEASUREMENT_ID: usize = 1;

#[derive(Debug)]
enum DBCommsError {
    DBConnection(influxdb::Error),
    QueryMeasurementID(influxdb::Error),
    DeserialiseMeasurementID(serde_json::Error),
    ParseMeasurementID(Value),
    ConvertUsize(u64),
    MissingTimeStamp,
    ParseTimeStamp(String),
}

/// Measurement / sensor data timeseries ID
/// Used in the table name of the current timeseries
/// Should correspond to the total number of recorded timeseries today + 1
type MeasurementID = usize;

type TableName = String;

/// Log recorded sensor data and logs to the database
pub async fn communicate_with_db(mut db_receiver: Receiver<SensorData>) {
    // Loop timekeeping
    let mut next_tick_time = Instant::now() + DB_LOOP_PERIOD;

    // Initialize DB connection
    let db_client = Arc::new(Client::new(DB_URI, DB_NAME).with_token(DB_ACCESS_TOKEN));

    // Initialize local state
    let sensor_data_table = initialize(db_client.clone()).await;
    let mut batched_data = Vec::with_capacity(QUERY_BATCH_LEN);
    let mut fall_back_storage = Vec::new();

    // Main routine
    loop {
        // Receive data from the microcontroller communication task
        if let Some(sensor_data) = db_receiver.recv().await {
            // Batch received measurements
            batched_data.push(SensorDataRecord::from(sensor_data));
            debug!("batched_query {:?}", batched_data);
        } else {
            error!("DB write error: unable to receive sensor date - Receiver is closed");
        }

        // Write measurements to DB when batch is filled
        if batched_data.len() >= QUERY_BATCH_LEN {
            let query: Vec<WriteQuery> = batched_data
                .clone()
                .into_iter()
                .map(|el| el.into_query(sensor_data_table.clone()))
                .collect();

            match db_client.query(query).await {
                Ok(_) => error!("Inserted Batched measurements into the DB"),
                Err(err) => {
                    error!(
                        "Error inserting batched measurements into the DB: {:?} - using fallback",
                        err
                    );

                    // Write to fallback hashmap
                    fall_back_storage.append(&mut batched_data);
                }
            }
            batched_data.clear();
        }

        // Loop timekeeping
        next_tick_time += DB_LOOP_PERIOD;
        time::sleep_until(next_tick_time).await;
    }
}

/// A Database record to track measurement ids
#[derive(Debug, influxdb::InfluxDbWriteable)]
struct MeasurementIDRecord {
    time: DateTime<Utc>,
    current: f64,
}

/// Initialize DB connection and table name to use for current measurement set
pub async fn initialize(db_client: Arc<Client>) -> TableName {
    let measurement_id = get_measurement_id(db_client).await;
    let date = Local::now().format("%Y_%m_%d").to_string();
    let sensor_data_table = format!("sensor_data_{}_{}", date, measurement_id);

    sensor_data_table
}

/// Fetches measurement id from the database, else use DEFAULT_MEASUREMENT_ID
pub async fn get_measurement_id(db_client: Arc<Client>) -> MeasurementID {
    let measurement_id = match fetch_measurement_id_from_db(db_client.clone()).await {
        Ok(id) => {
            // We are starting a new measurement timeseries, so increase id by one
            id + 1
        }
        Err(err) => {
            error!(
                "Problem fetching measurement id: {:?}, using default: {}",
                err, DEFAULT_MEASUREMENT_ID
            );
            DEFAULT_MEASUREMENT_ID
        }
    };

    // Save new measurement ID in DB
    write_measurement_id_to_db(measurement_id, db_client).await;

    error!("Starting new measurement number {}", measurement_id);
    measurement_id
}

/// Writes given measurement ID to the database
pub async fn write_measurement_id_to_db(id: MeasurementID, db_client: Arc<Client>) {
    let record = MeasurementIDRecord {
        time: Utc::now(),
        current: id as f64,
    };

    match db_client
        .query(record.into_query(MEASUREMENT_ID_TABLE))
        .await
    {
        Ok(msg) => info!("Inserted new measurement id into the DB: {:?}", msg),
        Err(err) => error!("Error inserting new measurement id into the DB: {:?}", err),
    }
}

/// Attempt to fetch measurement id from the database
pub async fn fetch_measurement_id_from_db(db_client: Arc<Client>) -> Result<usize, DBCommsError> {
    // Fetch latest measurement id and its timestamp
    let query = influxdb::ReadQuery::new(GET_LATEST_MEASUREMENT_ID);

    let json = db_client
        .query(query)
        .await
        .map_err(|err| DBCommsError::QueryMeasurementID(err))?;

    let v: Value =
        serde_json::from_str(&json).map_err(|err| DBCommsError::DeserialiseMeasurementID(err))?;

    // Parse latest measurement id timestamp
    let timestamp = &v["results"][0]["series"][0]["values"][0][0]
        .as_str()
        .ok_or_else(|| DBCommsError::MissingTimeStamp)?;
    let timestamp: DateTime<Utc> = timestamp
        .parse()
        .map_err(|_| DBCommsError::ParseTimeStamp(timestamp.to_string()))?;

    // Last recorded timeseries is yesterday or earlier, start a new measurement id
    if is_yesterday_or_earlier(timestamp) {
        info!("{:?} is yesterday or earlier, use default", timestamp);
        return Ok(DEFAULT_MEASUREMENT_ID);
    }

    // At least one timeseries was recorded today, parse measurement id
    let measurement_id = &v["results"][0]["series"][0]["values"][0][1];

    let measurement_id = measurement_id
        .as_f64()
        .ok_or_else(|| DBCommsError::ParseMeasurementID(measurement_id.clone()))?;

    Ok(measurement_id as usize)
}

/// Is the given date yesterday or earlier
fn is_yesterday_or_earlier(ts: DateTime<Utc>) -> bool {
    let now = Utc::now();
    let today_start = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();

    return ts < today_start;
}
