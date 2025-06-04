use chrono::{DateTime, TimeDelta, Utc};
use tokio::time::{self, Duration, Instant};
use tracing::{info, instrument};
use uom::si::{f32::Frequency, frequency::hertz};

use crate::{
    appstate::AppState,
    hardware::{MockloopHardware, Valve, ValveState},
};

/// Phases of the heart ventricles
/// Systole = ventricle contraction, Diastole = ventricle relaxation
#[derive(Debug)]
enum CardiacPhases {
    Systole,
    Diastole,
}

impl CardiacPhases {
    fn switch(&self) -> Self {
        match self {
            CardiacPhases::Systole => CardiacPhases::Diastole,
            CardiacPhases::Diastole => CardiacPhases::Systole,
        }
    }
}

/// Mockloop controller state machine states
#[derive(Debug)]
enum ControllerState {
    PreOp,
    Op,
    Err,
}

/// Controller for the Mockloop
#[derive(Debug)]
pub struct MockloopController<T: MockloopHardware> {
    // Mockloop hardware interface
    hw: T,
    // Current mockloop controller state
    state: ControllerState,
    // Control loop Frequency
    loop_frequency: Frequency,
    // Time at last cycle
    last_cycle_time: DateTime<Utc>,
    // Desired heart rate
    heart_rate: Frequency,
    // Current cardiac phase
    current_cardiac_phase: CardiacPhases,
    // Time spent in current cardiac phase
    time_spent_in_current_phase: TimeDelta,
    // Ratio of systole duration to total cardiac phase duration
    // NOTE: usually 3/7
    systole_ratio: f32,
}

impl<T> MockloopController<T>
where
    T: MockloopHardware,
{
    pub fn new(mut hw: T) -> Self {
        info!("Initialize controller");
        // Initialize hardware
        if let Err(err) = hw.initialize() {
            todo!("properly handle hardware initialize error: {err}");
        }

        MockloopController {
            state: ControllerState::PreOp,
            loop_frequency: Frequency::new::<hertz>(10.0),
            heart_rate: Frequency::new::<hertz>(80.0 / 60.0),
            last_cycle_time: Utc::now(),
            current_cardiac_phase: CardiacPhases::Systole,
            time_spent_in_current_phase: TimeDelta::zero(),
            systole_ratio: 3.0 / 7.0,
            hw,
        }
    }

    /// Run the MockloopController
    #[instrument(skip(app_state, self))]
    pub async fn run(mut self, app_state: AppState) {
        // Calculate the desired control loop interval
        let period: f32 = 1.0 / self.loop_frequency.get::<hertz>();
        let mut next_tick_time = Instant::now() + Duration::from_secs_f32(period);

        // Run the control loop
        loop {
            if !*app_state.enable_controller.lock().unwrap() {
                // Set control loop to pre operation while controller is disabled
                self.state = ControllerState::PreOp;
                // Make sure mockloop is in safe position
                self.hw.to_safe_state().unwrap();
            } else {
                // Controller enabled -> tick the controller state machine
                self.tick(app_state.clone()).await;
            }

            // Time bookkeeping
            self.last_cycle_time = Utc::now();
            // trace!("control looping")

            // Preempt until desired control loop interval has passed
            tokio::time::sleep_until(next_tick_time).await;

            let period: f32 = 1.0 / self.loop_frequency.get::<hertz>();
            next_tick_time += Duration::from_secs_f32(period);
        }
    }

    /// Single tick of the controller state machine
    pub async fn tick(&mut self, app_state: AppState) {
        match &self.state {
            ControllerState::PreOp => self.preop(),
            ControllerState::Op => self.op(app_state),
            ControllerState::Err => self.err(),
        };
    }

    /// Pre operation logic, actuate mockloop into safe state, reset cardiac phase time tracking
    /// and transition to Operational
    fn preop(&mut self) {
        info!(state = "PREOP");

        // Make sure the mockloop is in a safe state
        self.hw.to_safe_state().unwrap();

        // Reset the cardiac phase tracking
        self.current_cardiac_phase = CardiacPhases::Systole;
        self.time_spent_in_current_phase = TimeDelta::zero();

        self.state = ControllerState::Op;
    }

    /// Error state logic, unrecoverable
    fn err(&mut self) {
        info!(state = "ERR");

        // Make sure the mockloop is in a safe state
        self.hw.to_safe_state().unwrap();

        self.state = ControllerState::Err;
    }

    /// Operational logic, control ventricles and pressure regulator
    fn op(&mut self, app_state: AppState) {
        info!(state = "OP");
        self.control_ventricles(app_state.clone());
        self.control_pressure_regulator(app_state);
    }

    /// Set pressure regulator to the latest setpoint received for it
    fn control_pressure_regulator(&mut self, app_state: AppState) {
        // Blocking acquire the setpoint mutex and send the required regulator pressure, if this fails
        // we do nothing but hope the next state machine run correctly picks up the setpoint
        if let Ok(setpoint) = app_state.setpoint.lock() {
            info!(
                state = "OP",
                "Setting regulator pressure: {:?}", setpoint.controller_pressure_regulator
            );

            self.hw
                .set_regulator_pressure(setpoint.controller_pressure_regulator)
                .unwrap();
        } else {
            todo!("handle errors")
        }
    }

    /// Control the ventricle pneumatic valves in such a way the heart beats at the desired heartrate
    fn control_ventricles(&mut self, app_state: AppState) {
        // Time bookkeeping
        let current_time = Utc::now();
        self.time_spent_in_current_phase += current_time - self.last_cycle_time;
        info!(
            "time spent in current cardiac phase: {:?}",
            self.time_spent_in_current_phase
        );

        // Check if its time to switch cardiac phase
        let current_cardiac_phase_duration = TimeDelta::from_std(Duration::from_secs_f32(
            1.0 / self.heart_rate.get::<hertz>()
                * match self.current_cardiac_phase {
                    CardiacPhases::Systole => self.systole_ratio,
                    CardiacPhases::Diastole => 1.0 - self.systole_ratio,
                },
        ))
        .unwrap();

        info!(
            "Current phase duration {:?}",
            current_cardiac_phase_duration
        );

        // Switch cardiac phase when necessary
        if self.time_spent_in_current_phase > current_cardiac_phase_duration {
            self.current_cardiac_phase = self.current_cardiac_phase.switch();
            self.time_spent_in_current_phase = TimeDelta::zero();
        }

        // Actuate the ventricle valves according to the current cardiac phase
        let (left_valve_setpoint, right_valve_setpoint) = match self.current_cardiac_phase {
            CardiacPhases::Systole => (ValveState::Open, ValveState::Closed),
            CardiacPhases::Diastole => (ValveState::Closed, ValveState::Open),
        };

        info!("Setting left valve: {:?}", left_valve_setpoint);
        info!("Setting right valve: {:?}", right_valve_setpoint);
        self.hw.set_valve(Valve::Left, left_valve_setpoint).unwrap();
        self.hw
            .set_valve(Valve::Right, right_valve_setpoint)
            .unwrap();
    }
}
