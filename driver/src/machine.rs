use arrayvec::ArrayVec;
use library::{ArgumentMnumonic, CommandId, CommandMnumonics, GcodeCommand};

use crate::{my_clock::millis, stepper::{Speed, Stepper}, write_uart};

#[derive(Clone, Copy)]
pub enum Axis {X, Y, Z}

pub struct Machine<FRecvGcode, FStep, FDir>
where FRecvGcode : Fn() -> Option<GcodeCommand>,
    FStep: FnMut(Axis),
    FDir: FnMut(Axis, bool)
{
    pub x: Stepper<FStep, FDir>,
    pub y: Stepper<FStep, FDir>,
    pub z: Stepper<FStep, FDir>,
    next_command: FRecvGcode,
    command_buffer: ArrayVec<GcodeCommand, 4>
}

impl<F, FStep, FDir> Machine<F, FStep, FDir>
    where 
        F: Fn() -> Option<GcodeCommand>,
        FStep: FnMut(Axis) + Copy,
        FDir: FnMut(Axis, bool) + Copy
{
    pub fn new(next_command: F, step_fn: FStep, dir_fn: FDir, stepper_conf: [(Speed, u32); 3]) -> Self {
        Self {
            x: Stepper::new(Axis::X, step_fn.clone(),   dir_fn, stepper_conf[0].0.clone(), stepper_conf[0].1),
            y: Stepper::new(Axis::Y, step_fn.clone(),   dir_fn, stepper_conf[1].0.clone(), stepper_conf[1].1),
            z: Stepper::new(Axis::Z, step_fn,           dir_fn, stepper_conf[2].0.clone(), stepper_conf[2].1),
            command_buffer: Default::default(),
            next_command,
        }
    }

    fn setup_next_target(&mut self) {
        if let Some(command) = self.command_buffer.get(0) {
            match command.command_id {
                CommandId{ mnumonic: CommandMnumonics::G, major: 0, minor: 0 } => {
                    let mut min_acc = u32::MAX;
                    for arg in command.arguments.iter() {
                        let stepper = match arg.mnumonic {
                            ArgumentMnumonic::X => &mut self.x,
                            ArgumentMnumonic::Y => &mut self.y,
                            ArgumentMnumonic::Z => &mut self.z,
                        };
                        min_acc = min_acc.min(stepper.prog_speed.acceleration);
                        // TODO: relative vs abs.
                        // TODO: mm to step conversion with the fraction component.
                        //let numb_int = stepper.steps_per_mm as i32 * arg.value.major;
                        let numb_int = arg.value.major;

                        let mut buffer: str_buf::StrBuf<20> = str_buf::StrBuf::new();
                        ufmt::uwrite!(buffer, "g0 x[{}]\n", numb_int).unwrap();
                        write_uart(buffer.as_str());

                        stepper.set_target(numb_int.into())
                    }
                },
                _ => todo!("do no know how to process command."),
            }
        }
    }

    pub fn poll_task(&mut self) {
        let now = millis() as u64;
        if self.command_buffer.remaining_capacity() != 0 {
            if let Some(next) = (self.next_command)() {
                self.command_buffer.push(next);
                self.setup_next_target();
            }
        }
        if self.command_buffer.len() != 0 {
            let at_target = self.x.poll_task(now) && self.y.poll_task(now) && self.z.poll_task(now);
            if at_target {
                self.command_buffer.remove(0);
                self.setup_next_target();
            }
        }
    }
}
