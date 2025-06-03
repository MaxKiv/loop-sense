

    fn to_safe_state(&mut self) {
        set_regulator_pressure_setpoint(0.0)
        close_pressure_valve()
        open_vacuum_valve()
    }
