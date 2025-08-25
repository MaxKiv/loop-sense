use chrono::Utc;
use tokio::sync::watch::{Receiver, Sender};
use tracing::*;
use uuid::Uuid;

use crate::experiment::{Experiment, ExperimentStartMessage};

/// Backend of the "experiment manager" HHH frontend feature
/// Responsible for responding to experiment status changes, like starting or stopping an
/// experiment.
/// Also responsible for generating a new uuid when an experiment is started
pub async fn manage_experiments(
    mut experiment_started_receiver: Receiver<Option<ExperimentStartMessage>>,
    experiment_sender: Sender<Option<Experiment>>,
) {
    loop {
        // Wait untill a new experiment is started
        if experiment_started_receiver.changed().await.is_ok() {
            match experiment_started_receiver.borrow_and_update().clone() {
                Some(ExperimentStartMessage { name, description }) => {
                    // Construct a new experiment
                    let new_experiment = Experiment {
                        is_running: true,
                        id: Uuid::new_v4(),
                        name: name.to_string(),
                        description,
                        table_name: create_table_from_experiment(&name),
                        start_time: Utc::now(),
                        duration_seconds: chrono::Duration::zero(),
                    };

                    info!("New experiment started: {:?}", new_experiment);

                    // Notify control loop
                    experiment_sender.send(Some(new_experiment));
                }
                None => {
                    // Experiment stopped, notify control loop
                    experiment_sender.send(None);
                }
            }
        }
    }
}

fn create_table_from_experiment(name: &str) -> String {
    format!("experiment_{}_{}", name, Utc::now().format("%Y%m%d_%H%M%S"))
}
