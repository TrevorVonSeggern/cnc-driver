use arrayvec::ArrayVec;
use library::{CommandId, CommandMnumonics, GcodeCommand, XYZData, XYZId};

use crate::{recieve_gcode, stepper::{Speed, StepDir, Stepper}, write_uart};

const CLOCK_FACTOR: u32 = 1_000_000;
const MM_TICK_FACTOR: f32 = CLOCK_FACTOR as f32 / 100_000.0;

pub struct Machine<SD: StepDir>
{
    pub steppers: XYZData<Stepper<SD, CLOCK_FACTOR>>,
    step_resolution: XYZData<f32>,
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
            steppers: XYZData { x, y, z },
            step_resolution,
            command_buffer: Default::default(),
        }
    }

    fn setup_next_target(&mut self) {
        if let Some(command) = self.command_buffer.get(0) {
            match command.command_id {
                CommandId{ mnumonic: CommandMnumonics::G, major: 0, minor: 0 } => {
                    //let mut min_acc = u32::MAX;
                    for arg in command.arguments.iter() {
                        let id = XYZId::from_arg(arg.mnumonic);
                        if let Some(id) = id {
                            let stepper = self.steppers.match_id_mut(id);
                            //min_acc = min_acc.min(stepper.prog_speed.acceleration);
                            // TODO: relative vs abs.
                            // TODO: mm to step conversion with the fraction component.
                            //let numb_int = stepper.steps_per_mm as i32 * arg.value.major;
                            let numb_int = (arg.value.major as f32 * self.step_resolution.match_id(id)) as i32;
                            stepper.set_target(numb_int);
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
