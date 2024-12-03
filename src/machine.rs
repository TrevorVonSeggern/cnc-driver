use crate::stepper::Stepper;


#[derive(Default)]
pub struct Machine {
    pub x: Stepper,
    pub y: Stepper,
    pub z: Stepper,
}
