use chrono::Utc;
use tokio::sync::watch::{Receiver, Sender};
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
            if let Some(new_experiment_message) =
                experiment_started_receiver.borrow_and_update().clone()
            {
                // Construct a new experiment
                let new_experiment = Experiment {
                    is_running: true,
                    id: Uuid::new_v4(),
                    name: new_experiment_message.name.to_string(),
                    description: new_experiment_message.description.to_string(),
                    table_name: "TODO".to_string(),
                    start_time: Utc::now(),
                    duration_seconds: chrono::Duration::zero(),
                };

                // Notify control loop?
                experiment_sender.send(Some(new_experiment));
            }
        }
    }
}
