use influxdb::{Client, InfluxDbWriteable as _, WriteQuery};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, Duration};
use tracing::*;

use crate::control::ControllerReport;
use crate::database::secrets::*;
use crate::messages::db_messages::DatabaseRecord;

const DB_LOOP_PERIOD: Duration = Duration::from_millis(10);
const QUERY_BATCH_LEN: usize = 10;

#[derive(Debug)]
pub enum DBCommsError {
    DBConnection(influxdb::Error),
    QueryMeasurementID(influxdb::Error),
    DeserialiseMeasurementID(serde_json::Error),
    ParseMeasurementID(Value),
    ConvertUsize(u64),
    MissingTimeStamp,
    ParseTimeStamp(String),
}

/// Log recorded sensor data and logs to the database
pub async fn communicate_with_db(mut db_report_receiver: Receiver<ControllerReport>) {
    // Loop timekeeping
    let mut ticker = time::interval(DB_LOOP_PERIOD);

    // Initialize DB connection
    let db_client = Arc::new(Client::new(DB_URI, DB_NAME).with_token(DB_ACCESS_TOKEN));

    // Initialize local state
    let mut batched_data = Vec::with_capacity(QUERY_BATCH_LEN);
    let mut fall_back_storage = Vec::new();

    info!("initialized DB task, waiting for experiment start");

    // Main routine
    loop {
        // Wait to receive report from the controller task task:
        // This means an experiment is running and we need to log the measurements to the DB
        if let Some(report) = db_report_receiver.recv().await {
            // Batch received measurements
            batched_data.push(DatabaseRecord::from(report.clone()));
            info!("batched_query {:?}", batched_data);

            // Write measurements to DB when batch is filled
            if batched_data.len() >= QUERY_BATCH_LEN {
                let query: Vec<WriteQuery> = batched_data
                    .clone()
                    .into_iter()
                    .map(|el| el.into_query(report.experiment.table_name.clone()))
                    .collect();

                match db_client.query(query).await {
                    Ok(_) => info!("Inserted Batched measurements into the DB"),
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
        } else {
            error!(
                "DB write error: unable to receive report from controller task - Receiver is closed"
            );
        }

        // Loop timekeeping
        ticker.tick().await;
    }
}
