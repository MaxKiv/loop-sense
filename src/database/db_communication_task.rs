use influxdb::{Client, InfluxDbWriteable, WriteQuery};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::*;

use crate::control::ControllerReport;
use crate::database::secrets::*;
use crate::messages::db_messages::DatabaseRecord;

const QUERY_BATCH_LEN: usize = 100;

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
    // Initialize DB connection
    let db_client = Arc::new(loop {
        let client = Client::new(DB_URI, DB_NAME).with_token(DB_ACCESS_TOKEN);
        if let Ok(info) = client.ping().await {
            info!("DB found: {:?}", info);
            break client;
        }
    });

    // Initialize local state
    let mut batched_data: [DatabaseRecord; QUERY_BATCH_LEN] =
        core::array::from_fn(|_| DatabaseRecord::default());
    let mut write_idx: usize = 0;
    let mut fall_back_storage: Vec<DatabaseRecord> = Vec::new();

    info!("initialized DB task, waiting for experiment start");

    // Main routine
    loop {
        if !fall_back_storage.is_empty() {
            info!("fallback storage not empty, attempting to write to DB");
            let table_name = fall_back_storage[0].experiment_name.clone();
            let query = construct_write_query(&fall_back_storage, &table_name).await;

            // Attempt to write data to DB
            match db_client.query(query).await {
                Ok(_) => {
                    info!("Fallback storage DB write success");
                    fall_back_storage.clear();
                }
                Err(err) => {
                    error!(
                        "Unable to write fallback storage to DB: {:?}, retrying later...",
                        err
                    );
                }
            }
        }

        // Wait to receive report from the controller task task:
        // This means an experiment is running and we need to log the measurements to the DB
        if let Some(report) = db_report_receiver.recv().await {
            // Batch received measurements
            let table_name = report.experiment.table_name.clone();
            let db_record = DatabaseRecord::from(report);
            info!(
                "Batched DatabaseRecord {:?}, total # batched {:?}",
                db_record, write_idx
            );
            batched_data[write_idx] = db_record;

            // Write measurements to DB when batch is filled
            if write_idx == (QUERY_BATCH_LEN - 1) {
                let query = construct_write_query(&batched_data, &table_name).await;

                // Attempt to write data to DB
                match db_client.query(query).await {
                    Ok(_) => info!("Inserted Batched measurements into the DB"),
                    Err(err) => {
                        error!(
                            "Error inserting batched measurements into the DB: {:?} - using fallback",
                            err
                        );

                        // Write to fallback hashmap
                        fall_back_storage.push(batched_data[write_idx].clone());
                    }
                }

                write_idx = 0;
            }
        } else {
            error!(
                "DB write error: unable to receive report from controller task - Receiver is closed"
            );
        }
    }
}

async fn construct_write_query(
    batched_data: &[DatabaseRecord],
    table_name: &str,
) -> Vec<WriteQuery> {
    let query: Vec<WriteQuery> = batched_data
        .iter()
        .map(|el| {
            el.clone()
                .try_into_query(table_name)
                .expect("Unable to create query")
        })
        .collect();

    query
}
