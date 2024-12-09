use panic_halt as _;
use stepgen::Stepgen;

#[derive(Default)]
pub struct StepperConfig {
    pub steps_per_mm: f32,
    pub max_feed_rate: f32, // in units mm / s
    pub max_acceleration: f32, // in units mm / s^2
}

#[derive(Default)]
pub struct StepperStats {
    pub position: i64,
    pub max_feed_rate: u32, // in units mm / s
    pub acceleration: u32, // in units mm / s^2
    pub direction: bool, // true for positive
}

#[derive(Default)]
pub struct StepperPins {
    pub step: u32,
    pub dir: u32,
    pub enable: bool,
}

pub struct Stepper {
    pub config: StepperConfig,
    pub stats: StepperStats,
    pub pins: StepperPins,
    stepper: stepgen::Stepgen,
    next_delay: Option<u32>,
    next_vel: u32,
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

impl Stepper {
    pub fn new() -> Self {
        return Self {
            config: StepperConfig::default(),
            stats: StepperStats::default(),
            pins: StepperPins::default(),
            //stepper: Stepgen::new(1_000_000),
            stepper: default_stepgen(),
            next_delay: None,
            next_vel: 0,
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

    pub fn next_step_time(&self) -> (Option<u32>, u32) {
        return (self.next_delay, self.next_vel);
    }

    fn calc_next_step(&mut self) {
        self.next_delay = self.stepper.next().map(|t| t >> 8);
        self.next_vel = self.stepper.current_speed();
    }

    pub fn set_target(&mut self, target_step: i64) {
        let target_distance = (target_step - self.stats.position).abs();
        self.reset_stepgen();
        self.stepper.set_target_step(target_distance as u32);
        self.calc_next_step();
    }

    pub fn set_position(&mut self, position: i64) {
        self.stats.position = position;
    }

    pub fn step(&mut self) {
        self.calc_next_step();
        self.stats.position = self.stats.position + 1;
        // todo: callback for setting steps and direction.
    }
}
