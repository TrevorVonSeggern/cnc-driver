use library::{StepIterator, XYZId};

#[derive(Default, Clone)]
pub struct Speed<T> {
    pub speed: T, // in units mm / s. 8bits for dec.
    pub acceleration: u32, // in units mm / s^2. 8bits for dec.
}

impl<T> Speed<T> {
    pub fn map<FS, FA, TN>(self, speed_fn: FS, acc_fn: FA) -> Speed<TN>
        where FS: Fn(T) -> TN, FA : Fn(u32) -> u32
    {
        Speed::<TN> { speed: speed_fn(self.speed), acceleration: acc_fn(self.acceleration) }
    }
}

#[derive(Clone, Default)]
pub struct StepperTiming {
    pub next_update_time: u64,
    pub off_at: u64,
    pub delay_duration: u32,
}

const SIGNAL_LENGTH: u64 = 10;

impl StepperTiming {
    pub fn update_needed(&self, now: u64) -> bool {
        self.next_update_time != 0 && now >= self.next_update_time
    }

    pub fn is_uninitialized(&self) -> bool {
        self.next_update_time == 0
    }

    pub fn update(&mut self, delay: u32, now: Option<&u64>) {
        self.delay_duration = delay;
        if delay == 0 {
            self.next_update_time = 0;
            self.off_at = 0;
        }
        else if self.next_update_time != 0 {
            self.off_at = self.next_update_time + SIGNAL_LENGTH;
            self.next_update_time = self.next_update_time + delay as u64;
        }
        else {
            self.off_at = *now.unwrap() + SIGNAL_LENGTH;
            self.next_update_time = *now.unwrap() + delay as u64;
        }
    }
}

pub trait StepDir: Clone {
    fn step(&self, axis: XYZId);
    fn dir(&self, axis: XYZId, direction: bool);
}

pub struct Stepper<SD: StepDir> {
    axis: XYZId,
    step_dir_fn: SD,
    timing: StepperTiming,
    step_iter: library::StepIterator,
}

impl<SD: StepDir> Stepper<SD>{
    pub fn new(axis: XYZId, step_dir_fn: SD, acc_table: &'static [u32]) -> Self {
        return Self {
            axis,
            step_dir_fn,
            timing: Default::default(),
            step_iter: StepIterator::new(acc_table),
        };
    }

    pub fn set_target(&mut self, target_step: i32, speed: u32) {
        let slew_delay_us = 1_000_000 / speed as u32;
        self.step_iter.set_target(target_step, slew_delay_us, 0);
        self.step_dir_fn.dir(self.axis, self.step_iter.direction.is_negative());
    }

    fn step(&mut self) {
        self.step_dir_fn.step(self.axis);
        self.timing.update(self.step_iter.next().unwrap_or(0), None);
    }

    pub fn on_target(&self) -> bool {
        self.step_iter.target == self.step_iter.position
    }

    pub fn poll_task(&mut self, now: u64) {
        if self.on_target() {}
        else if self.timing.is_uninitialized() { // first step calc.
            self.timing.update(self.step_iter.next().unwrap_or(0), Some(&now));
        }
        else if self.timing.off_at != 0 && now >= self.timing.off_at {
            self.step_dir_fn.step(self.axis);
            self.timing.off_at = 0;
        }
        else if self.timing.update_needed(now) {
            self.step();
        }
    }

    pub fn stop(&mut self) {
        //self.timing = Default::default();
    }

    pub fn get_target(&self) -> i32 {
        self.step_iter.target
    }
}
