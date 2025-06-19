pub const GET_LATEST_MEASUREMENT_ID: &str =
    "SELECT * FROM measurement_id ORDER BY time DESC LIMIT 1;";
