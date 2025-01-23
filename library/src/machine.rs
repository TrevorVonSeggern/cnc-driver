use arrayvec::ArrayVec;
use crate::{ArgumentMnumonic, CanRecieve, CommandId, CommandMnumonics, GcodeCommand, StepDir, Stepper, XYZData, XYZId, ACC_CURVE, RESOLUTION, STEPPER_SPEED};

pub enum AbsMode {
    Abs,
    Relative,
}

pub struct Machine<SD: StepDir>
{
    pub steppers: XYZData<Stepper<SD>>,
    //motor_max_speed: XYZData<u32>,
    max_feed_rate: u32,
    feed_rate: u32,
    home_offset: XYZData<i32>,
    command_buffer: ArrayVec<GcodeCommand, 2>,
    abs_mode: AbsMode,
}

pub const RES_F32: f32 = RESOLUTION as f32;

#[allow(static_mut_refs)]
impl<SD: StepDir> Machine<SD>
{
    pub fn new(step_dir_fn: SD) -> Self {
        let x = Stepper::new(XYZId::X, step_dir_fn.clone(), ACC_CURVE.as_ref());
        let y = Stepper::new(XYZId::Y, step_dir_fn.clone(), ACC_CURVE.as_ref());
        let z = Stepper::new(XYZId::Z, step_dir_fn.clone(), ACC_CURVE.as_ref());
        Self {
            feed_rate: STEPPER_SPEED * RESOLUTION,
            max_feed_rate: STEPPER_SPEED * RESOLUTION,
            //motor_max_speed: speeds,
            steppers: XYZData { x, y, z },
            command_buffer: Default::default(),
            home_offset: Default::default(),
            abs_mode: AbsMode::Abs,
        }
    }

    fn move_command(&mut self, target: XYZData<Option<i32>>, speed: u32) {
        if target.all(|v| v.is_none()) || speed == 0 {
            return;
        }
        if let Some(x) = target.x {
            self.steppers.x.set_target(x, speed);
        }
        //let abs_offset = match self.abs_mode {
            //AbsMode::Relative => Default::default(),
            //AbsMode::Abs => XYZData {
                //x: self.steppers.x.get_position() + self.home_offset.z,
                //y: self.steppers.y.get_position() + self.home_offset.y,
                //z: self.steppers.z.get_position() + self.home_offset.x
            //},
        //};
        //let displacement = XYZData {
            //x: target.x.map(|x| x - abs_offset.x).unwrap_or_default(),
            //y: target.y.map(|y| y - abs_offset.y).unwrap_or_default(),
            //z: target.z.map(|z| z - abs_offset.z).unwrap_or_default(),
        //};
        //let move_distance = u32sqrt(displacement.iter().map(|v| (v * v) as u32).sum());
        //if move_distance == 0 {
            //return;
        //}
        //let move_distance = move_distance as f32;
        //let altered_speed_fn = |distance: i32| -> u32 {
            //(speed as f32 * (distance as f32 / move_distance)) as u32
        //};
        //if displacement.x != 0 {
            //self.steppers.x.set_target(displacement.x + abs_offset.x, altered_speed_fn(displacement.x));
        //}
        //if displacement.y != 0 {
            //self.steppers.y.set_target(displacement.y + abs_offset.y, altered_speed_fn(displacement.y));
        //}
        //if displacement.z != 0 {
            //self.steppers.z.set_target(displacement.z + abs_offset.z, altered_speed_fn(displacement.z));
        //}
    }

    fn feed_argument<'a>(&self, args: &GcodeCommand) -> Option<u32> {
        let args = args.arguments.iter();
        args.filter(|a| a.mnumonic == ArgumentMnumonic::F).next().map(|a| a.value.major as u32)
    }

    fn setup_next_target(&mut self) {
        if let Some(command) = self.command_buffer.get(0) {
            match command.command_id {
                CommandId{ mnumonic: CommandMnumonics::G, major: 0, minor: 0 } => {
                    let mut target = XYZData::<Option<i32>>::default();
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            *target.match_id_mut(id) = Some((arg.value.float * RES_F32) as i32);
                        }
                    }
                    self.feed_rate = self.feed_argument(&command).unwrap_or(self.feed_rate);
                    self.move_command(target, self.max_feed_rate.clone());
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 1, minor: 0 } => {
                    let mut target = XYZData::<Option<i32>>::default();
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            *target.match_id_mut(id) = Some((arg.value.float * RES_F32) as i32);
                        }
                    }
                    self.feed_rate = self.feed_argument(&command).unwrap_or(self.feed_rate);
                    self.move_command(target, self.feed_rate.clone());
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 9, minor: 2 } => {
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            *self.home_offset.match_id_mut(id) = (arg.value.float * RES_F32) as i32;
                        }
                    }
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 9, minor: 0 } => {
                    self.abs_mode = AbsMode::Abs;
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 9, minor: 1 } => {
                    self.abs_mode = AbsMode::Relative;
                },
                _ => todo!("do no know how to process command."),
            }
        }
        else {
            //let mut buffer: str_buf::StrBuf<100> = str_buf::StrBuf::new();
            //ufmt::uwriteln!(buffer, "timing for x: {}, {}\n", self.steppers.x.timing.next_update_time, self.steppers.x.timing.delay_duration).unwrap();
            //write_uart(buffer.as_str());
        }
    }

    pub fn poll_task(&mut self, reciever: &impl CanRecieve<GcodeCommand>) {
        if self.command_buffer.remaining_capacity() != 0 {
            if let Some(next) = reciever.recieve() {
                self.command_buffer.push(next);
                if self.command_buffer.len() == 1 {
                    self.setup_next_target();
                }
            }
        }
        if self.command_buffer.len() != 0 && self.steppers.all(|s| s.on_target()) {
            self.command_buffer.remove(0);
            self.setup_next_target();
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

#[cfg(test)]
mod tests {
    use super::*;
}
