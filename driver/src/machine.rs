use core::fmt::Arguments;

use arrayvec::ArrayVec;
use library::{CommandArgument, CommandId, CommandMnumonics, GcodeCommand, XYZData, XYZId};

use crate::{recieve_gcode, stepper::{Speed, StepDir, Stepper}, write_uart};

const CLOCK_FACTOR: u32 = 1_000_000;
const MM_TICK_FACTOR: f32 = CLOCK_FACTOR as f32 / 100_000.0;

pub struct Machine<SD: StepDir>
{
    pub steppers: XYZData<Stepper<SD, CLOCK_FACTOR>>,
    motor_max_speed: XYZData<Speed<u32>>,
    max_feed_rate: Speed<u32>,
    feed_rate: Speed<u32>,
    home_offset: XYZData<i32>,
    pub step_resolution: XYZData<f32>,
    command_buffer: ArrayVec<GcodeCommand, 4>
}

impl Speed<f32> {
    pub fn scale_speed_per_res(self, steps_per_mm: f32) -> Speed<u32> {
        self.map(|s| (s * steps_per_mm as f32 * MM_TICK_FACTOR) as u32, |a| (a as f32 * steps_per_mm * MM_TICK_FACTOR) as u32)
    }
}

impl<SD: StepDir> Machine<SD>
{
    pub fn new(step_dir_fn: SD, stepper_conf: XYZData<Speed<f32>>, step_resolution: XYZData<f32>) -> Self {
        let x = Stepper::new(XYZId::X, step_dir_fn.clone(), stepper_conf.x.scale_speed_per_res(step_resolution.x));
        let y = Stepper::new(XYZId::Y, step_dir_fn.clone(), stepper_conf.y.scale_speed_per_res(step_resolution.y));
        let z = Stepper::new(XYZId::Z, step_dir_fn.clone(), stepper_conf.z.scale_speed_per_res(step_resolution.z));
        Self {
            motor_max_speed: XYZData { x: x.speed.clone(), y: y.speed.clone(), z: z.speed.clone() },
            feed_rate: x.speed.clone(),
            max_feed_rate: x.speed.clone(),
            steppers: XYZData { x, y, z },
            step_resolution,
            command_buffer: Default::default(),
            home_offset: Default::default(),
        }
    }

    fn move_command(&mut self, mut target: XYZData<Option<i32>>, speed: Speed<u32>) {
        target.x = target.x.map(|x| x + self.home_offset.x);
        target.y = target.x.map(|y| y + self.home_offset.y);
        target.z = target.x.map(|z| z + self.home_offset.z);
        for (target, stepper) in target.iter().zip(self.steppers.iter_mut()).filter_map(|(t, s)| t.map(|ss| (ss, s))) {
            stepper.set_target(target);
            stepper.speed = speed.clone();
        }
    }

    fn feed_argument<'a>(&self, args: &GcodeCommand) -> Option<u32> {
        let args = args.arguments.iter();
        args.filter(|a| a.mnumonic == library::ArgumentMnumonic::F).next().map(|a| a.value.major as u32)
    }

    fn setup_next_target(&mut self) {
        if let Some(command) = self.command_buffer.get(0) {
            match command.command_id {
                CommandId{ mnumonic: CommandMnumonics::G, major: 0, minor: 0 } => {
                    let mut target = XYZData::<Option<i32>>::default();
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            let resolution = self.step_resolution.match_id(id);
                            *target.match_id_mut(id) = Some((arg.value.major as f32 * resolution) as i32);
                        }
                    }
                    self.feed_rate.speed = self.feed_argument(&command).unwrap_or(self.feed_rate.speed);
                    self.move_command(target, self.max_feed_rate.clone());
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 1, minor: 0 } => {
                    let mut target = XYZData::<Option<i32>>::default();
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            let resolution = self.step_resolution.match_id(id);
                            *target.match_id_mut(id) = Some((arg.value.major as f32 * resolution) as i32);
                        }
                    }
                    self.feed_rate.speed = self.feed_argument(&command).unwrap_or(self.feed_rate.speed);
                    self.move_command(target, self.feed_rate.clone());
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 9, minor: 2 } => {
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            let resolution = self.step_resolution.match_id(id);
                            *self.home_offset.match_id_mut(id) = (arg.value.major as f32 * resolution) as i32;
                        }
                    }
                },
                _ => todo!("do no know how to process command."),
            }
        }
        else {
            write_uart("no next move. Full stop for motors.\n");
            //let _ = self.steppers.iter_mut().map(|s| s.stop());
        }
    }

    pub fn poll_task(&mut self) {
        if self.command_buffer.remaining_capacity() != 0 {
            if let Some(next) = recieve_gcode() {
                self.command_buffer.push(next);
                if self.command_buffer.len() == 1 {
                    self.setup_next_target();
                }
            }
        }
        if self.command_buffer.len() != 0 {
            if self.steppers.all(|s| s.on_target()) {
                self.command_buffer.remove(0);
                self.setup_next_target();
            }
        }
    }
    pub fn step_monitor(&mut self, now: u64, axis: XYZId) {
        if self.command_buffer.len() != 0 {
            // poll only one axis at a time for 'niceness'. This code executes in
            // interrupts so we don't want other interrupts for timekeeping to be missed.
            self.steppers.one_map_mut(axis, |s| s.poll_task(now));
        }
    }
}
