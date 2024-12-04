use panic_halt as _;
use library::physics_acceleration;

#[derive(Default)]
pub struct StepperConfig {
    pub steps_per_mm: f32,
    pub max_feed_rate: f32, // in units mm / s
    pub max_acceleration: f32, // in units mm / s^2
}

#[derive(Default)]
pub struct StepperStats {
    pub position: u32,
    pub max_feed_rate: f32, // in units mm / s
    pub instant_feed_rate: f32, // in units mm / s
    pub acceleration: f32, // in units mm / s^2
    pub last_update: u64, // in millis
    pub direction: bool, // true for positive
}

#[derive(Default)]
pub struct StepperPins {
    pub step: u32,
    pub dir: u32,
    pub enable: bool,
}

#[derive(Default)]
pub struct Stepper {
    pub config: StepperConfig,
    pub stats: StepperStats,
    pub pins: StepperPins,
}

impl Stepper {
    pub fn next_step_time(&self) -> Option<(u64, f32)> {
        //let distance = (position - self.stats.position) as i64;
        //if position == self.stats.position {
            //return None;
        //}
        // decelerating
        //let deceleration_distance = (self.stats.instant_feed_rate as f32 / (-2.0 * self.stats.acceleration as f32)) as i64;
        //if deceleration_distance >= distance {
        //}
        // maintaining speed
        //if self.stats.instant_feed_rate >= self.stats.max_feed_rate {
            //let step_time = (self.config.steps_per_mm as f32 / self.stats.max_feed_rate as f32) as u64;
            //return Some(self.stats.last_update + step_time);
        //}
        // accelerating
        //else {
        //let a = self.stats.acceleration;
        let a = 0.1;
        let vi = self.stats.instant_feed_rate;
        //let vi = 0.447213595;
        let (t, vf) = physics_acceleration(a, vi, 1.0);
        return Some(((t * 1000.0) as u64, vf));
        //}
    }

    pub fn step(&mut self, now: u64, feed_rate: f32) {
        //let time = (now - self.stats.last_update) as f32 / 1000.0;
        //self.stats.last_update = now;
        //self.stats.instant_feed_rate = feed_rate; // 1 step / time
        self.stats.position = self.stats.position + 1;
        //let max_fr = (self.stats.max_feed_rate * 100.0) as i32;
        //if max_fr >= 0 {
            //if !self.stats.direction {
                //self.stats.direction = true;
                //// todo, also trigger the pin.
            //}
            //if self.stats.last_update != now {
            //}
        //}
        //else {
            //if self.stats.direction {
                //self.stats.direction = false;
            //}
            //self.stats.position = self.stats.position - 1;
        //}
    }
}
