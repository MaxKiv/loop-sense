use axum::extract::FromRef;
use chrono::{DateTime, TimeDelta, Utc};
use tokio::time::{self, Duration};
use uom::si::{f32::Frequency, frequency::hertz, time::millisecond};

use crate::{appstate::AppState, hardware::MockloopHardware};

/// Phases of the heart ventricles
/// Systole = ventricle contraction, Diastole = ventricle relaxation
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
enum ControllerState {
    PREOP,
    OP,
    ERR,
}

/// Controller for the Mockloop
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
    // Ratio of systole to total cardiac phase
    // NOTE: usually 3/7
    systole_ratio: f32,
}

impl<T: MockloopHardware> MockloopController<T> {
    /// Initialize and run the MockloopController
    pub async fn run(mut self, app_state: AppState) {
        // Initialize hardware
        if let Err(err) = self.hw.initialize() {
            todo!("properly handle hardware initialize error: {err}");
        }

        // Initialize controller defaults
        self.state = ControllerState::PREOP;
        self.loop_frequency = Frequency::new::<hertz>(100.0);
        self.heart_rate = Frequency::new::<hertz>(60.0);
        self.last_cycle_time = Utc::now();
        self.current_cardiac_phase = CardiacPhases::Systole;
        self.time_spent_in_current_phase = TimeDelta::zero();
        self.systole_ratio = 3.0 / 7.0;

        // Run the control loop
        loop {
            // Calculate the desired control loop interval
            let period: f32 = 1.0 / self.loop_frequency.get::<hertz>();
            let mut control_loop_interval = time::interval(Duration::from_secs_f32(period));

            if !*app_state.enable_controller.lock().unwrap() {
                // Set control loop to pre operation while controller is disabled
                self.state = ControllerState::PREOP;
            } else {
                // Controller enabled -> tick the controller state machine
                self.tick(app_state.clone());
            }

            // Time bookkeeping
            self.last_cycle_time = Utc::now();
            // Preempt until desired control loop interval has passed
            control_loop_interval.tick().await;
            println!("control looping")
        }
    }

    /// Single tick of the controller state machine
    pub async fn tick(&mut self, app_state: AppState) {
        match &self.state {
            ControllerState::PREOP => self.preop(),
            ControllerState::OP => self.op(app_state),
            ControllerState::ERR => self.err(),
        };
    }

    /// Pre operation logic, actuate mockloop into safe state, reset cardiac phase time tracking
    /// and transition to Operational
    fn preop(&mut self) {
        // Make sure the mockloop is in a safe state
        self.hw.to_safe_state();

        // Reset the cardiac phase tracking
        self.current_cardiac_phase = CardiacPhases::Systole;
        self.time_spent_in_current_phase = TimeDelta::zero();

        self.state = ControllerState::OP;
    }

    /// Error state logic, unrecoverable
    fn err(&mut self) {
        // Make sure the mockloop is in a safe state
        self.hw.to_safe_state();

        self.state = ControllerState::ERR;
    }

    /// Operational logic, control ventricles and pressure regulator
    fn op(&mut self, app_state: AppState) {
        self.control_ventricles(app_state.clone());
        self.control_pressure_regulator(app_state);
    }

    /// Set pressure regulator to the latest setpoint received for it
    fn control_pressure_regulator(&mut self, app_state: AppState) {
        // Blocking acquire the setpoint mutex and send the required regulator pressure, if this fails
        // we do nothing but hope the next state machine run correctly picks up the setpoint
        if let Ok(setpoint) = app_state.setpoint.lock() {
            self.hw
                .set_regulator_pressure(setpoint.controller_pressure_regulator);
        } else {
            todo!("handle errors")
        }
    }

    /// Control the ventricle pneumatic valves in such a way the heart beats at the desired heartrate
    fn control_ventricles(&mut self, app_state: AppState) {
        // Time bookkeeping
        let current_time = Utc::now();
        self.time_spent_in_current_phase += current_time - self.last_cycle_time;

        // Check if its time to switch cardiac phase
        let current_cardiac_phase_duration = TimeDelta::from_std(Duration::from_secs_f32(
            self.heart_rate.get::<hertz>()
                * match self.current_cardiac_phase {
                    CardiacPhases::Systole => self.systole_ratio,
                    CardiacPhases::Diastole => 1.0 - self.systole_ratio,
                },
        ))
        .unwrap();

        // Switch cardiac phase when necessary
        if self.time_spent_in_current_phase > current_cardiac_phase_duration {
            self.current_cardiac_phase = self.current_cardiac_phase.switch()
        }

        // Actuate the ventricle valves according to the current cardiac phase
        match self.current_cardiac_phase {
            CardiacPhases::Systole => {}
            CardiacPhases::Diastole => {}
        }

        // match self.current_cardiac_phase:
        //     case CardiacPhases.SYSTOLE:
        //         # print("Systole")
        //         self.open_pressure_valve()
        //         self.close_vacuum_valve()
        //     case CardiacPhases.DIASTOLE:
        //         # print("Diastole")
        //         self.close_pressure_valve()
        //         self.open_vacuum_valve()
    }
}
