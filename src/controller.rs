/// High level control loop for the HHH SBC, responsible for:
/// * Parsing received MCU reports
///     - Calculating cardiac output
///     - Sending report to DB task
///     - Populating Appstate::latest_report
/// * Constructing setpoints for the MCU / low level controller
///     -
/// * Reacting to experiment changes
///     - Starting heart controller on new experiment
///     - Stopping heart contyroller on end/stop of experiment
pub async fn control_loop(mut db_receiver: Receiver<SensorData>) {}
