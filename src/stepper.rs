use panic_halt as _;
#[allow(unused_imports)]
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
        // maintaining speed
        if self.stats.instant_feed_rate >= self.stats.max_feed_rate {
            let step_time = (self.config.steps_per_mm as f32 / self.stats.max_feed_rate as f32) as u64;
            return Some(self.stats.last_update + step_time);
        }
        // accelerating
        else {
            let a = self.stats.acceleration as f32;
            let vi = self.stats.instant_feed_rate as f32;
            let d = 1.0 / self.config.steps_per_mm as f32;
            let t = (((2.0*a*d) + (vi*vi)).sqrt() + vi) / a;
            return Some(t as u64 + self.stats.last_update);
        }
    }

    pub fn step(&mut self, now: u64) {
        let max_fr = (self.stats.max_feed_rate * 100.0) as i32;
        if max_fr >= 0 {
            if !self.stats.direction {
                self.stats.direction = true;
                // todo, also trigger the pin.
            }
            if self.stats.last_update != now {
                let time = (now - self.stats.last_update) as f32;
                self.stats.instant_feed_rate = 1.0 / time; // 1 step / time
            }
            self.stats.position = self.stats.position + 1;
        }
        else {
            if self.stats.direction {
                self.stats.direction = false;
            }
            self.stats.position = self.stats.position - 1;
        }
        self.stats.last_update = now;
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(0.0), 0.0);
        assert_eq!(sqrt(2.0), 1.0);
        assert_eq!(sqrt(4.0), 2.0);
        assert_eq!(sqrt(9.0), 3.0);
        assert!(sqrt(15.0) - 3.87298 < 1e-4); // sqrt(15) is approximately 3.87298
        assert_eq!(sqrt(16.0), 4.0);
        assert!(sqrt(26.0) - 5.099 < 1e-3); // sqrt(26) is approximately 5.099
    }
}
