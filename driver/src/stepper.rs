use panic_halt as _;
use stepgen::Stepgen;

use crate::{machine::Axis, write_uart};

#[derive(Default, Clone)]
pub struct Speed {
    pub speed: u32, // in units mm / s. 8bits for dec.
    pub acceleration: u32, // in units mm / s^2. 8bits for dec.
    pub decceleration: u32, // in units mm / s^2. 8bits for dec.
}

#[derive(Default)]
pub struct StepperConfig {
    pub steps_per_mm: u32,
    pub max_speed: Speed,
}

//#[derive(Default)]
//pub struct StepperStats {
    //pub position: i64,
    //pub max_feed_rate: u32, // in units mm / s
    //pub acceleration: u32, // in units mm / s^2
    //pub direction: bool, // true for positive
//}

pub struct Stepper<FStep, FDir> where FStep: FnMut(Axis), FDir: FnMut(Axis, bool) {
    axis: Axis,
    step_fn: FStep,
    dir_fn: FDir,

    pub steps_per_mm: u32,
    pub max_speed: Speed,
    pub prog_speed: Speed,
    pub current_move_speed: Speed,

    //pub pins: StepperPins,
    stepper: stepgen::Stepgen,
    position: i64,
    target: i64,
    next_delay: Option<u32>,
    next_update: Option<u64>,
}

fn default_stepgen() -> stepgen::Stepgen {
    Stepgen::new(1_000)
}

pub fn float_to_u32_fraction(float: f32) -> u32 {
    return (float as u32) << 8; // TODO: the fractional part.
    //let integer = float.floor() as u32;
    //let m = float * 
    //let fraction = float - integer;
    //return (integer << 8) | fraction;
}

impl<FStep, FDir> Stepper<FStep, FDir> where FStep: FnMut(Axis), FDir: FnMut(Axis, bool) {
    pub fn new(axis: Axis, step_fn: FStep, dir_fn: FDir, stepper_speed: Speed, steps_per_mm: u32) -> Self {
        return Self {
            axis,
            step_fn,
            dir_fn,
            steps_per_mm,
            max_speed: stepper_speed.clone(),
            prog_speed: stepper_speed.clone(),
            current_move_speed: stepper_speed,
            //pins: StepperPins::default(),
            //stepper: Stepgen::new(1_000_000),
            stepper: default_stepgen(),
            position: 0,
            target: 0,
            next_delay: None,
            next_update: None,
        };
    }

    fn reset_stepgen(&mut self) {
        self.stepper = default_stepgen();
        self.stepper.set_acceleration(5 << 8).unwrap();
        self.stepper.set_target_speed(10 << 8).unwrap();
    }
    //fn reset_stepgen(&mut self) -> Result<&mut Self, &'static str> {
        //self.stepper = default_stepgen();
        //self.stepper.set_acceleration(5 << 8).map_err(|e| match e { 
            //stepgen::Error::TooSlow => "Acceleration too slow",
            //stepgen::Error::TooFast => "Acceleration too fast",
            //_ => "other error",
        //})?;
        //self.stepper.set_target_speed(10 << 8).map_err(|e| match e { 
            //stepgen::Error::TooSlow => "Speed too slow.",
            //stepgen::Error::TooFast => "Speed too fast.",
            //_ => "other error",
        //})?;
        //return Ok(self);
    //}

    fn calc_next_step(&mut self, now: u64) {
        self.next_delay = self.stepper.next().map(|t| t >> 8);
        self.next_update = self.next_delay.map(|d| d as u64 + now);
    }

    pub fn set_target(&mut self, target_step: i64) {
        self.target = target_step;
        let v = target_step - self.position;
        (self.dir_fn)(self.axis, v.is_negative());
        let target_distance = v.abs() as u32;
        self.reset_stepgen();
        let _ = self.stepper.set_target_step(target_distance as u32).map_err(|_| {
            write_uart("Unable to set target step\n");
        });
    }

    #[allow(unused)]
    pub fn set_position(&mut self, position: i64) {
        self.position = position;
    }

    fn step(&mut self) {
        if (self.target - self.position).is_positive() {
            self.position += 1;
        }
        else {
            self.position -= 1;
        }
        (self.step_fn)(self.axis);
    }

    pub fn poll_task(&mut self, now: u64) -> bool {
        if self.target == self.position {
            return true;
        }
        if self.next_update.is_none() {
            self.calc_next_step(now);
        }
        if let Some(next_time) = self.next_update {
            if next_time <= now {
                self.step();
                self.calc_next_step(now);
            }
        }
        return self.target == self.position;
    }
}
