use crate::experiment::ExperimentStatus;
use crate::http::messages::{ExperimentList, ExperimentListFromDB, ExperimentFromDB};
use crate::{AxumState, http::messages::HeartbeatMessage, messages::frontend_messages::Report};
use crate::database::secrets::*;
use axum::Json;
use axum::http::StatusCode;
use serde_json::Value;
use tracing::*;

/// Returns the latest measurement report from the mcu
#[axum::debug_handler]
pub async fn get_measurements(
    state: axum::extract::State<AxumState>,
) -> Result<Json<Report>, StatusCode> {
    if let Ok(guard) = state.report.lock() {
        if let Some(ref report) = *guard {
            // Return json-serialised report
            info!("GET measurements returning report: {:?}", report);
            return Ok(Json(report.clone()));
        } else {
            // No report was generated yet
            warn!("GET measurements attempted but no report in axum state");
            return Err(StatusCode::NO_CONTENT);
        }
    }

    // Unable to lock the axum state mutex_guard, shit definitely hit the fan
    error!(
        "Unable to lock the report mutex guard during GET measurements, returning INTERNAL_SERVER_ERROR and moving on with life..."
    );
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Return a heartbeat message
#[axum::debug_handler]
pub async fn get_heartbeat(_state: axum::extract::State<AxumState>) -> Json<HeartbeatMessage> {
    Json(HeartbeatMessage::new())
}

/// Return status of the currently running experiment
#[axum::debug_handler]
pub async fn get_experiment_status(
    state: axum::extract::State<AxumState>,
) -> Result<Json<ExperimentStatus>, StatusCode> {
    if let Ok(experiment) = state.current_experiment.lock() {
        if let Some(ref exp) = *experiment {
            // Convert to ExperimentStatus on-the-fly to get fresh duration calculation
            let status: ExperimentStatus = exp.into();
            Ok(Json(status))
        } else {
            Err(StatusCode::NO_CONTENT)
        }
    } else {
        error!("Unable to fetch the current experiment");
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Return all experiments from in-memory state
#[axum::debug_handler]
pub async fn get_list_experiment(
    state: axum::extract::State<AxumState>,
) -> Result<Json<ExperimentList>, StatusCode> {
    let experiments = state.experiments.lock();

    if let Ok(experiments) = experiments {
        let experiments = experiments.clone();
        info!("Returning experiment list: {:?}", experiments);
        return Ok(Json(experiments));
    }

    warn!("GET experiment list returned nothing");
    Err(StatusCode::NO_CONTENT)
}

/// Return all experiments by querying InfluxDB directly
#[axum::debug_handler]
pub async fn get_list_experiments_from_db(
    _state: axum::extract::State<AxumState>,
) -> Result<Json<ExperimentListFromDB>, StatusCode> {
    match query_experiments_from_influxdb().await {
        Ok(experiments) => {
            info!("Returning {} experiments from database", experiments.len());
            Ok(Json(ExperimentListFromDB { experiments }))
        }
        Err(e) => {
            error!("Failed to retrieve experiments from InfluxDB: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Query InfluxDB for all experiment tables and their metadata
async fn query_experiments_from_influxdb() -> Result<Vec<ExperimentFromDB>, String> {
    // Create HTTP client
    let client = reqwest::Client::new();

    // Query to list all tables starting with 'experiment_' in the iox schema
    let list_tables_query = r#"
        SELECT DISTINCT table_name 
        FROM information_schema.tables 
        WHERE table_name LIKE 'experiment_%' AND table_schema = 'iox'
        ORDER BY table_name DESC
    "#;

    // Execute query to get table names
    let url = format!("{}/api/v3/query_sql", DB_URI);
    info!("Querying InfluxDB at: {}", url);
    info!("Query: {}", list_tables_query);
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", DB_ACCESS_TOKEN))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "db": DB_NAME,
            "q": list_tables_query,
            "format": "json"
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to query InfluxDB: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("InfluxDB query failed with status {}: {}", status, error_text);
        return Err(format!("InfluxDB query failed with status: {}", status));
    }

    let tables_result: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse InfluxDB response: {}", e))?;

    info!("InfluxDB response: {}", serde_json::to_string_pretty(&tables_result).unwrap_or_else(|_| "Failed to serialize".to_string()));

    // Extract table names from response
    let table_names = extract_table_names(&tables_result)
        .ok_or_else(|| format!("Failed to extract table names from response: {:?}", tables_result))?;

    info!("Found {} experiment tables: {:?}", table_names.len(), table_names);

    if table_names.is_empty() {
        return Ok(Vec::new());
    }

    // For each table, query first and last record to get metadata
    let mut experiments = Vec::new();
    for table_name in table_names {
        match get_experiment_metadata(&client, &table_name).await {
            Ok(Some(experiment)) => experiments.push(experiment),
            Ok(None) => {
                warn!("No metadata found for table: {}", table_name);
            }
            Err(e) => {
                error!("Failed to get metadata for table {}: {}", table_name, e);
                // Continue with other tables instead of failing completely
            }
        }
    }

    Ok(experiments)
}

/// Extract table names from InfluxDB query response
fn extract_table_names(response: &Value) -> Option<Vec<String>> {
    // InfluxDB 3.0 returns results as an array of objects: [{"table_name": "..."}]
    if let Some(results) = response.as_array() {
        let tables: Vec<String> = results
            .iter()
            .filter_map(|result| {
                result.get("table_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        
        info!("Extracted {} table names from response", tables.len());
        if !tables.is_empty() {
            return Some(tables);
        }
    } else {
        warn!("Response is not an array: {:?}", response);
    }

    None
}

/// Get metadata for a specific experiment table
async fn get_experiment_metadata(
    client: &reqwest::Client,
    table_name: &str,
) -> Result<Option<ExperimentFromDB>, String> {
    // Query first record
    let first_query = format!(
        r#"SELECT experiment_id, experiment_name, experiment_description, time
           FROM "{}"
           ORDER BY time ASC
           LIMIT 1"#,
        table_name
    );

    // Query last record
    let last_query = format!(
        r#"SELECT time
           FROM "{}"
           ORDER BY time DESC
           LIMIT 1"#,
        table_name
    );

    let url = format!("{}/api/v3/query_sql", DB_URI);

    // Get first record
    let first_response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", DB_ACCESS_TOKEN))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "db": DB_NAME,
            "q": first_query,
            "format": "json"
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to query first record: {}", e))?;

    if !first_response.status().is_success() {
        return Err(format!("First record query failed: {}", first_response.status()));
    }

    let first_data: Value = first_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse first record: {}", e))?;

    // Get last record
    let last_response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", DB_ACCESS_TOKEN))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "db": DB_NAME,
            "q": last_query,
            "format": "json"
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to query last record: {}", e))?;

    if !last_response.status().is_success() {
        return Err(format!("Last record query failed: {}", last_response.status()));
    }

    let last_data: Value = last_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse last record: {}", e))?;

    // Extract metadata from responses
    let first_record = extract_first_record(&first_data);
    let last_time = extract_last_time(&last_data);

    if let (Some(first), Some(last_time)) = (first_record, last_time) {
        // InfluxDB timestamps may not have timezone suffix, add 'Z' if missing
        let start_time_str = if first.start_time.ends_with('Z') {
            first.start_time.clone()
        } else {
            format!("{}Z", first.start_time)
        };
        let end_time_str = if last_time.ends_with('Z') {
            last_time.clone()
        } else {
            format!("{}Z", last_time)
        };

        // Calculate duration
        let start_time = chrono::DateTime::parse_from_rfc3339(&start_time_str)
            .map_err(|e| format!("Failed to parse start time '{}': {}", start_time_str, e))?;
        let end_time = chrono::DateTime::parse_from_rfc3339(&end_time_str)
            .map_err(|e| format!("Failed to parse end time '{}': {}", end_time_str, e))?;
        
        let duration_seconds = (end_time - start_time).num_milliseconds() as f64 / 1000.0;

        Ok(Some(ExperimentFromDB {
            table_name: table_name.to_string(),
            experiment_id: first.experiment_id,
            experiment_name: first.experiment_name,
            description: first.description,
            start_time: Some(first.start_time),
            duration_seconds,
        }))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
struct FirstRecordData {
    experiment_id: String,
    experiment_name: String,
    description: String,
    start_time: String,
}

/// Extract first record data from query response
fn extract_first_record(response: &Value) -> Option<FirstRecordData> {
    // InfluxDB returns array format: [{"experiment_id": "...", "experiment_name": "...", ...}]
    if let Some(results) = response.as_array() {
        if let Some(first) = results.first() {
            let experiment_id = first.get("experiment_id")?.as_str()?.to_string();
            let experiment_name = first.get("experiment_name")?.as_str()?.to_string();
            let description = first.get("experiment_description").and_then(|d| d.as_str()).unwrap_or("").to_string();
            let start_time = first.get("time")?.as_str()?.to_string();
            
            info!("Extracted first record: id={}, name={}, time={}", experiment_id, experiment_name, start_time);
            return Some(FirstRecordData {
                experiment_id,
                experiment_name,
                description,
                start_time,
            });
        } else {
            warn!("First record array is empty");
        }
    } else {
        warn!("First record response is not an array: {:?}", response);
    }

    None
}

/// Extract last time from query response
fn extract_last_time(response: &Value) -> Option<String> {
    // InfluxDB returns array format: [{"time": "..."}]
    if let Some(results) = response.as_array() {
        if let Some(last) = results.first() {
            let time = last.get("time")?.as_str()?.to_string();
            info!("Extracted last time: {}", time);
            return Some(time);
        } else {
            warn!("Last time array is empty");
        }
    } else {
        warn!("Last time response is not an array: {:?}", response);
    }

    None
}
