use crate::{StepIterator, XYZId};

#[derive(Clone, Default)]
pub struct StepperTiming {
    pub next_update_time: u64,
}

const SIGNAL_LENGTH: u32 = 30;

impl StepperTiming {
    pub fn update_needed(&self, now: u64) -> bool {
        self.next_update_time != 0 && now >= self.next_update_time
    }

    pub fn is_uninitialized(&self) -> bool {
        self.next_update_time == 0
    }

    pub fn uninit(&mut self) {
        self.next_update_time = 0;
    }

    pub fn update(&mut self, delay: u32) {
        self.next_update_time = self.next_update_time + delay as u64;
    }
}

pub trait StepDir: Clone {
    fn step(&mut self, axis: XYZId);
    fn dir(&mut self, axis: XYZId, direction: bool);
}

pub struct Stepper<SD: StepDir> {
    axis: XYZId,
    step_dir_fn: SD,
    cycle_high: bool,
    pub timing: StepperTiming,
    pub step_iter: StepIterator,
}

impl<SD: StepDir> Stepper<SD>{
    pub fn new(axis: XYZId, step_dir_fn: SD, acc_table: &'static [u32]) -> Self {
        return Self {
            axis,
            step_dir_fn,
            cycle_high: false,
            timing: Default::default(),
            step_iter: StepIterator::new(acc_table),
        };
    }

    pub fn on_target(&self) -> bool { self.get_target() == self.get_position() && self.timing.is_uninitialized() }
    pub fn get_target(&self) -> i32 { self.step_iter.target }
    pub fn get_position(&self) -> i32 { self.step_iter.position }

    pub fn set_target(&mut self, target_step: i32, speed: u32) {
        let slew_delay_us = 1_000_000 / speed as u32;
        self.step_iter.set_target(target_step, slew_delay_us, 0);
        self.step_dir_fn.dir(self.axis, self.step_iter.direction.is_negative());
    }

    fn step(&mut self) {
        self.step_dir_fn.step(self.axis);
        if !self.cycle_high {
            self.cycle_high = true;
            self.timing.update(SIGNAL_LENGTH);
        }
        else if let Some(delay) = self.step_iter.next() {
            self.timing.update((delay.saturating_sub(SIGNAL_LENGTH)).max(SIGNAL_LENGTH));
            self.cycle_high = false;
        }
        else {
            self.timing.uninit();
            self.cycle_high = false;
        }
    }

    pub fn poll_task(&mut self, now: u64) {
        if self.timing.update_needed(now) {
            self.step();
        }
        else if self.timing.is_uninitialized() && !self.on_target() { // first step calc.
            self.timing.next_update_time = now;
            self.timing.update(self.step_iter.next().unwrap_or(0));
            self.cycle_high = false;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_init_then_uninit() {
        let mut timing = StepperTiming::default();
        assert_eq!(timing.is_uninitialized(), true);
        timing.update(10);
        assert_eq!(timing.is_uninitialized(), false);
        timing.uninit();
        assert_eq!(timing.is_uninitialized(), true);
    }

    #[test]
    fn timing_updates() {
        let mut timing = StepperTiming::default();
        assert_eq!(timing.update_needed(0), false);
        assert_eq!(timing.update_needed(1000), false);
        timing.update(110);
        assert_eq!(timing.update_needed(99), false);
        assert_eq!(timing.update_needed(100), false);
        assert_eq!(timing.update_needed(109), false);
        assert_eq!(timing.update_needed(110), true);
        assert_eq!(timing.update_needed(111), true);
    }

    #[test]
    fn timing_updates_two_add() {
        let mut timing = StepperTiming::default();
        assert_eq!(timing.update_needed(0), false);
        assert_eq!(timing.update_needed(1000), false);
        timing.update(100);
        timing.update(10);
        assert_eq!(timing.update_needed(99), false);
        assert_eq!(timing.update_needed(100), false);
        assert_eq!(timing.update_needed(109), false);
        assert_eq!(timing.update_needed(110), true);
        assert_eq!(timing.update_needed(111), true);
    }

    #[derive(Default, Clone, Copy, Debug)]
    struct CounterStepper {
        pub current_step: u32,
        pub current_dir: bool,
    }
    impl StepDir for CounterStepper {
        fn step(&mut self, _: XYZId) { self.current_step += 1; }
        fn dir(&mut self, _: XYZId, direction: bool) { self.current_dir = direction; }
    }

    static ACC_TABLE: &[u32] = &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1 ];

    #[test]
    fn stepper_one_step_ever_step() {
        let mut stepper = Stepper::<CounterStepper>::new(XYZId::X, CounterStepper::default(), ACC_TABLE);
        stepper.step();
        assert_eq!(stepper.step_dir_fn.current_step, 1);
        assert_eq!(stepper.step_dir_fn.current_dir, false);
    }

    #[test]
    fn stepper_one_step() {
        let mut stepper = Stepper::<CounterStepper>::new(XYZId::X, CounterStepper::default(), ACC_TABLE);
        assert_eq!(stepper.on_target(), true);
        stepper.set_target(1, 10_000); // 1 step every 100.
        assert_eq!(stepper.on_target(), false);
        // first check should set the first update time, but not increase step counter.
        stepper.poll_task(100); // start at 100 time.
        assert_eq!(stepper.step_dir_fn.current_step, 0);
        assert_eq!(stepper.timing.next_update_time, 200);
        assert_eq!(stepper.cycle_high, false);
        // check early, don't step
        stepper.poll_task(110);
        assert_eq!(stepper.step_dir_fn.current_step, 0);
        assert_eq!(stepper.timing.next_update_time, 200);
        assert_eq!(stepper.cycle_high, false);
        // at time for step, should not be at target because of off signal length pulse time.
        stepper.poll_task(200);
        assert_eq!(stepper.step_dir_fn.current_step, 1);
        assert_eq!(stepper.step_dir_fn.current_dir, false);
        assert_eq!(stepper.cycle_high, true);
        assert_eq!(stepper.timing.next_update_time as u32, 200 + SIGNAL_LENGTH);
        assert_eq!(stepper.on_target(), false);
        // toggle pin off, should clear time.
        stepper.poll_task(200 + SIGNAL_LENGTH as u64);
        assert_eq!(stepper.step_dir_fn.current_step, 2);
        assert_eq!(stepper.step_dir_fn.current_dir, false);
        assert_eq!(stepper.cycle_high, false);
        assert_eq!(stepper.timing.is_uninitialized(), true);
        assert_eq!(stepper.on_target(), true);
        assert_eq!(stepper.step_iter.acc_iteration, 0);
    }

    #[test]
    fn stepper_two_steps() {
        let mut stepper = Stepper::<CounterStepper>::new(XYZId::X, CounterStepper::default(), ACC_TABLE);
        assert_eq!(stepper.on_target(), true);
        stepper.set_target(2, 10_000); // 1 step every 100.
        stepper.poll_task(100); // start at 100 time.
        stepper.poll_task(200); // first rising edge.
        stepper.poll_task(230); // first falling edge.
        stepper.poll_task(300); // second rising edge.
        stepper.poll_task(330); // second falling edge.
        assert_eq!(stepper.step_dir_fn.current_step, 4);
        assert_eq!(stepper.cycle_high, false);
        assert_eq!(stepper.on_target(), true);
        assert_eq!(stepper.timing.is_uninitialized(), true);
    }

    #[test]
    fn stepper_faster_than_signal_length() {
        let mut stepper = Stepper::<CounterStepper>::new(XYZId::X, CounterStepper::default(), ACC_TABLE);
        assert_eq!(stepper.on_target(), true);
        stepper.set_target(2, 100_000); // 1 step every 10.
        stepper.poll_task(100);
        stepper.poll_task(100u64 + (SIGNAL_LENGTH * 1) as u64); // requested 10, bumped to 30 for signal length.
        stepper.poll_task(100u64 + (SIGNAL_LENGTH * 2) as u64); // falling edge
        stepper.poll_task(100u64 + (SIGNAL_LENGTH * 3) as u64); // last rising edge.
        stepper.poll_task(100u64 + (SIGNAL_LENGTH * 4) as u64); // falling edge
        assert_eq!(stepper.step_dir_fn.current_step, 4);
        assert_eq!(stepper.cycle_high, false);
        assert_eq!(stepper.on_target(), true);
    }

    #[test]
    fn stepper_step_loop() {
        let mut stepper = Stepper::<CounterStepper>::new(XYZId::X, CounterStepper::default(), ACC_TABLE);
        let mut total_time = 0;
        for i in 0..10 { // move loop
            let target = if i % 2 == 0 { 10 } else { 0 };
            stepper.set_target(target, 100_000); // 1 step every 10.
            for _ in 0..100 { // poll loop
                total_time += 100;
                stepper.poll_task(total_time);
                //let time_diff = stepper.timing.next_update_time.saturating_sub(total_time);

                if stepper.on_target() {
                    break;
                }
            }
            assert_eq!(stepper.on_target(), true, "Should end on target.");
            assert_eq!(stepper.cycle_high, false, "Cycle should end on low (false).");
            assert_eq!(stepper.step_iter.acc_iteration, 0, "iteration should be 0 for acc dec between moves.");
            assert_eq!(stepper.timing.is_uninitialized(), true, "Time should be 0'ed between moves.");
        }
    }
}
