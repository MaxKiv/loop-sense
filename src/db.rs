use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use uom::si::{pressure::bar, volume_rate::liter_per_minute};

use crate::controller::backend::mockloop_hardware::SensorData;

// Default InfluxDB3 DB data
pub const DB_URI: &str = "http://localhost:8181";
pub const DB_NAME: &str = "test";
pub const DB_ACCESS_TOKEN: &str =
    "apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw";

#[derive(Debug, InfluxDbWriteable)]
pub struct SensorDataRecord {
    time: DateTime<Utc>,
    pressure_systemic_preload: f64,
    pressure_systemic_afterload: f64,
    controller_regulator_actual_pressure: f64,
    systemic_flow: f64,
    pulmonary_flow: f64,
}

impl From<SensorData> for SensorDataRecord {
    fn from(data: SensorData) -> Self {
        Self {
            time: data.experiment_time,
            pressure_systemic_preload: data.pressure_systemic_preload.get::<bar>(),
            pressure_systemic_afterload: data.pressure_systemic_afterload.get::<bar>(),
            controller_regulator_actual_pressure: data
                .controller_regulator_actual_pressure
                .get::<bar>(),
            systemic_flow: data.systemic_flow.get::<liter_per_minute>(),
            pulmonary_flow: data.pulmonary_flow.get::<liter_per_minute>(),
        }
    }
}
