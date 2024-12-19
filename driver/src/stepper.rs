use library::XYZId;

use crate::pins::write_uart;

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
    fn output(&self, axis: XYZId) -> bool;
}

pub struct Stepper<SD: StepDir, const CLOCK_FACTOR: u32> {
    axis: XYZId,
    step_dir_fn: SD,
    pub speed: Speed<u32>,
    direction: i8,
    position: i32,
    pub target: i32,
    acceleration_iteration: u8,
    timing: StepperTiming,
    slew_delay: u32,
}

impl<SD: StepDir, const CLOCK_FACTOR: u32> Stepper<SD, CLOCK_FACTOR>{
    pub fn new(axis: XYZId, step_dir_fn: SD, stepper_speed: Speed<u32>) -> Self {
        return Self {
            axis,
            step_dir_fn,
            speed: stepper_speed,
            slew_delay: 0,
            position: 0,
            target: 0,
            acceleration_iteration: 0,
            direction: 0,
            timing: Default::default(),
        };
    }

    pub fn set_target(&mut self, target_step: i32) {
        self.target = target_step;
        let displacement = target_step - self.position;
        self.direction = displacement.clamp(-1, 1) as i8;
        self.step_dir_fn.dir(self.axis, displacement.is_negative());
        self.slew_delay = CLOCK_FACTOR / self.speed.speed as u32;
        self.acceleration_iteration = 0;
        let mut buffer: str_buf::StrBuf<100> = str_buf::StrBuf::new();
        ufmt::uwrite!(buffer, "set target {} from {}\n", self.target, self.position).unwrap();
        write_uart(buffer.as_str());
    }

    fn step(&mut self) {
        self.position += self.direction as i32;
        self.step_dir_fn.step(self.axis);

        let mut delay = self.timing.delay_duration;
        if ((self.target - self.position).abs() + 1) <= self.acceleration_iteration as i32 {
            delay = library::inter_step_dec_delay(self.timing.delay_duration, self.acceleration_iteration as u32) + 1;
            self.acceleration_iteration = self.acceleration_iteration.saturating_sub(1);
            if self.acceleration_iteration == 0 {
                delay = 0;
            }
        }
        else if self.timing.delay_duration != self.slew_delay {
            self.acceleration_iteration = self.acceleration_iteration.saturating_add(1);
            delay = library::inter_step_acc_delay(self.timing.delay_duration, self.acceleration_iteration as u32);
            if delay <= self.slew_delay {
                delay = self.slew_delay;
            }
        }

        self.timing.update(delay, None);
    }

    pub fn on_target(&self) -> bool {
        self.target == self.position
    }

    pub fn poll_task(&mut self, now: u64) {
        if self.on_target() {}
        else if self.timing.next_update_time == 0 { // first step calc.
            let first_step = library::first_step_delay::<CLOCK_FACTOR>(self.speed.acceleration).max(self.slew_delay);
            self.timing.update(first_step.into(), Some(&now));
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
}
