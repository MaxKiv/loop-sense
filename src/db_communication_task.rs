use std::num::ParseIntError;

use chrono::{Local, Utc};
use influxdb::{Client, InfluxDbWriteable as _};
use loop_sense::controller::backend::mockloop_hardware::SensorData;
use loop_sense::database::query::GET_CURRENT_SEQUENCE;
use loop_sense::database::record::SensorDataRecord;
use loop_sense::database::secrets::{DB_ACCESS_TOKEN, DB_BUCKET, DB_NAME, DB_URI, TABLE_SEQUENCE};
use serde_json::Value;
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, Duration, Instant};
use tracing::{error, warn};

const DB_LOOP_PERIOD_MS: Duration = Duration::from_millis(100);
const QUERY_BATCH_LEN: usize = 100;
const DEFAULT_SEQUENCE: usize = 1;

#[derive(Debug)]
enum DBCommsError {
    DBConnection(influxdb::Error),
    QuerySequence(influxdb::Error),
    DeserialiseSequence(serde_json::Error),
    ParseSequence(Value),
}

pub async fn communicate_with_db(mut db_receiver: Receiver<SensorData>) {
    // Timekeeping
    let mut next_tick_time = Instant::now() + DB_LOOP_PERIOD_MS;

    let (db_client, table_name) = initialize().await;

    let mut batched_query = Vec::with_capacity(QUERY_BATCH_LEN);
    loop {
        if let Some(sensor_data) = db_receiver.recv().await {
            batched_query.push(SensorDataRecord::from(sensor_data).into_query(table_name.clone()));
            warn!("batched_query {:?}", batched_query);
        } else {
            warn!("DB write error: unable to receive sensor date - Receiver is closed");
        }

        if batched_query.len() >= QUERY_BATCH_LEN {
            match db_client.query(batched_query.clone()).await {
                Ok(msg) => warn!("DB client insert query return: {:?}", msg),
                Err(err) => warn!("DB client insert query return: {:?}", err),
            }
            batched_query.clear();
        }

        // Loop timekeeping
        next_tick_time += DB_LOOP_PERIOD_MS;
        time::sleep_until(next_tick_time).await;
    }
}

pub async fn initialize() -> (Client, String) {
    // Get sequence number
    let sequence = match calculate_sequence_number().await {
        Ok(sequence) => {
            warn!("fetched sequence: {}", sequence);
            sequence
        }
        Err(err) => {
            error!(
                "Problem fetching measurement sequence: {:?}, using default: {}",
                err, DEFAULT_SEQUENCE
            );
            DEFAULT_SEQUENCE
        }
    };

    let date = Local::now().format("%Y-%m-%d").to_string();
    let table_name = format!("sensor_data_{}_{}", date, sequence);

    let db_client = Client::new(DB_URI, DB_NAME).with_token(DB_ACCESS_TOKEN);

    (db_client, table_name)
}

pub async fn calculate_sequence_number() -> Result<usize, DBCommsError> {
    let db_client = Client::new(DB_URI, DB_NAME).with_token(DB_ACCESS_TOKEN);

    let query = influxdb::ReadQuery::new(GET_CURRENT_SEQUENCE);

    let json = db_client
        .query(query)
        .await
        .map_err(|err| DBCommsError::QuerySequence(err))?;

    let v: Value =
        serde_json::from_str(&json).map_err(|err| DBCommsError::DeserialiseSequence(err))?;
    let value = &v["results"][0]["series"][0]["values"][0][1];

    let sequence: usize = value
        .as_u64()
        .ok_or_else(|| return Err(DBCommsError::ParseSequence(value)));

    Ok(sequence)
}
