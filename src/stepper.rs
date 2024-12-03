//#[allow(unused_imports)]
use micromath::F32Ext;

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
    pub fn next_step_time(&self, position: u32) -> Option<u64> {
        let distance = (position - self.stats.position) as i64;
        if distance == 0 {
            return None;
        }
        // decelerating
        let deceleration_distance = (self.stats.instant_feed_rate as f32 / (-2.0 * self.stats.acceleration as f32)) as i64;
        if deceleration_distance >= distance {
        }
        if self.stats.instant_feed_rate >= self.stats.max_feed_rate {
            let step_time = (self.config.steps_per_mm as f32 / self.stats.max_feed_rate as f32) as u64;
            return Some(self.stats.last_update + step_time);
        }
        else {
            let a = self.stats.acceleration as f32;
            let vi = self.stats.instant_feed_rate as f32;
            let d = 1.0 / self.config.steps_per_mm as f32;
            let t = (((2.0*a*d) + (vi*vi)).sqrt() + vi) / a;
            return Some(t as u64 + self.stats.last_update);
        }
        //return 0;

        // accelerating
        // maintaining speed
            // at position
            //0 => self.stats.last_update,
    }

    pub fn step(&mut self, now: u64) {
        let max_fr = (self.stats.max_feed_rate * 100.0) as i32;
        if max_fr >= 0 {
            if !self.stats.direction {
                self.stats.direction = true;
                // todo, also trigger the pin.
            }
            if self.stats.last_update != now {
                let last_update = (now - self.stats.last_update) as f32;
                let d = 1.0 / 1.0;
                let f = d as f32;
                //self.stats.instant_feed_rate = f;

                //self.stats.instant_feed_rate = 1.0 / 1.0;
            }
        }
        self.stats.position = self.stats.position + 1;
        self.stats.last_update = now;
    }
}
