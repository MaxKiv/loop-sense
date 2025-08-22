use tokio::sync::mpsc::Sender;

use crate::experiment::Experiment;

/// Backend of the "experiment manager" HHH frontend feature
/// Responsible for responding to experiment status changes, like starting or stopping an
/// experiment.
/// Also responsible for generating a new uuid when an experiment is started
pub async fn manage_experiments(experiment_sender: Sender<Experiment>) {}
