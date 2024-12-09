use crate::stepper::Stepper;

pub struct Machine {
    pub x: Stepper,
    pub y: Stepper,
    pub z: Stepper,
}

impl Machine {
    pub fn new() -> Self {
        return Self {
            x: Stepper::new(),
            y: Stepper::new(),
            z: Stepper::new(),
        };
    }
}
