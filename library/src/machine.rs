use arrayvec::ArrayVec;
use crate::{u32sqrt, ArgumentMnumonic, CanRecieve, CommandId, CommandMnumonics, GcodeCommand, StepDir, Stepper, XYZData, XYZId, ACC_CURVE, RESOLUTION, STEPPER_SPEED};

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
        let current_position = self.steppers.map(|s| s.get_position());
        let move_vector = match self.abs_mode {
            AbsMode::Relative => target,
            AbsMode::Abs => target + self.home_offset - current_position.clone(),
        }.map(|p| p.unwrap_or_default());
        let move_distance = u32sqrt(move_vector.iter().map(|v| (v * v) as u32).sum());
        if move_distance == 0 {
            return;
        }
        let move_distance = move_distance as f64;
        let altered_speed_fn = |distance: i32| -> u32 {
            (speed as f64 * (distance.abs() as f64 / move_distance)) as u32
        };
        if move_vector.x != 0 {
            self.steppers.x.set_target(move_vector.x + current_position.x, altered_speed_fn(move_vector.x));
        }
        if move_vector.y != 0 {
            self.steppers.y.set_target(move_vector.y + current_position.y, altered_speed_fn(move_vector.y));
        }
        if move_vector.z != 0 {
            self.steppers.z.set_target(move_vector.z + current_position.z, altered_speed_fn(move_vector.z));
        }
    }

    fn feed_argument<'a>(args: &GcodeCommand) -> Option<u32> {
        let args = args.arguments.iter();
        args.filter(|a| a.mnumonic == ArgumentMnumonic::F).next().map(|a| ((a.value.float * 60.0 / (RESOLUTION as f32)) as u32))
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
                    let feed_rate = Self::feed_argument(&command).unwrap_or(self.max_feed_rate);
                    self.move_command(target, feed_rate);
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 1, minor: 0 } => {
                    let mut target = XYZData::<Option<i32>>::default();
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            *target.match_id_mut(id) = Some((arg.value.float * RES_F32) as i32);
                        }
                    }
                    self.feed_rate = Self::feed_argument(&command).unwrap_or(self.feed_rate);
                    self.move_command(target, self.feed_rate);
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 9, minor: 2 } => {
                    for arg in command.arguments.iter() {
                        if let Some(id) = XYZId::from_arg(arg.mnumonic) {
                            *self.home_offset.match_id_mut(id) = (arg.value.float * RES_F32) as i32;
                        }
                    }
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 90, minor: _ } => {
                    self.abs_mode = AbsMode::Abs;
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 91, minor: _ } => {
                    self.abs_mode = AbsMode::Relative;
                },
                CommandId{ mnumonic: CommandMnumonics::G, major: 21, minor: _ } => {},
                CommandId{ mnumonic: CommandMnumonics::G, major: 20, minor: _ } => { todo!("Inch mode not supported.")},
                CommandId{ mnumonic: CommandMnumonics::M, major: 30, minor: _ } => { 
                    self.feed_rate = self.max_feed_rate;
                },
                CommandId{ mnumonic: CommandMnumonics::M, major: _, minor: _ } => {},
                _ => { /*todo!("do no know how to process command.")*/ },
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
    use crate::*;

    use super::*;
    #[derive(Default, Clone, Copy, Debug)]
    struct CounterStepper {
        pub current_step: u32,
        pub current_dir: bool,
    }
    impl StepDir for CounterStepper {
        fn step(&mut self, _: XYZId) { self.current_step += 1; }
        fn dir(&mut self, _: XYZId, direction: bool) { self.current_dir = direction; }
    }

    #[test]
    pub fn machine_can_init() {
        let gcode_channel = SplitChannel::new(crate::Channel::<GcodeCommand, 3>::default());
        let gcode_input = gcode_channel.create_sender();
        let mut gcode: GcodeCommand = Default::default();
        gcode.command_id = CommandId{ mnumonic: CommandMnumonics::G, major: 0, minor: 0 };
        let mut x_arg: CommandArgument = Default::default();
        x_arg.mnumonic = ArgumentMnumonic::X;
        x_arg.value.float = 1.23;
        x_arg.value.major = 1;
        x_arg.value.minor = 23;
        gcode.arguments.push(x_arg);
        let _ = gcode_input.send(gcode);
        let mut machine = Machine::new(CounterStepper::default());
        machine.poll_task(&gcode_channel);
        machine.step_monitor(1, XYZId::X);
        let x_first_time = machine.steppers.x.timing.next_update_time.clone() as u32;
        assert_eq!(x_first_time, ACC_CURVE[0] + 1, "Straight move. First delay in acc curve.");
        assert!(!machine.steppers.x.on_target(), "Move requires movement.");
    }

    fn move_command(axis: XYZId, f:f32) -> GcodeCommand {
        let mut gcode: GcodeCommand = Default::default();
        gcode.command_id = CommandId{ mnumonic: CommandMnumonics::G, major: 1, minor: 0 };
        let arg: CommandArgument = CommandArgument {
            mnumonic: axis.into(),
            value: MajorMinorNumber {
                major: f as i32, minor: 0, float: f,
            },
        };
        gcode.arguments.push(arg);
        gcode
    }

    #[test]
    pub fn machine_forward_and_back() {
        let gcode_channel = SplitChannel::new(crate::Channel::<GcodeCommand, 3>::default());
        let gcode_input = gcode_channel.create_sender();
        let gcode_x1: GcodeCommand = move_command(XYZId::X, 1.0);
        let gcode_x0: GcodeCommand = move_command(XYZId::X, 0.0);
        let mut machine = Machine::new(CounterStepper::default());

        let _ = gcode_input.send(gcode_x1);
        machine.poll_task(&gcode_channel);
        machine.step_monitor(1, XYZId::X);
        assert!(!machine.steppers.x.on_target(), "Move requires movement. 1");
        for i in 2..100000 {
            machine.step_monitor(i * 10, XYZId::X);
            if machine.steppers.x.on_target() {
                break;
            }
        }
        assert!(machine.steppers.x.on_target(), "should be on target 10.");

        let _ = gcode_input.send(gcode_x0);
        machine.poll_task(&gcode_channel);
        machine.step_monitor(1, XYZId::X);
        assert!(!machine.steppers.x.on_target(), "Move requires movement. 0");
        for i in 2..100000 {
            machine.step_monitor(i * 10, XYZId::X);
            if machine.steppers.x.on_target() {
                break;
            }
        }
        assert!(machine.steppers.x.on_target(), "should be on target 0.");
    }

    #[test]
    pub fn machine_xy() {
        let gcode_channel = SplitChannel::new(crate::Channel::<GcodeCommand, 3>::default());
        let gcode_input = gcode_channel.create_sender();
        let mut gcode: GcodeCommand = move_command(XYZId::X, 1.0);
        gcode.arguments.push(move_command(XYZId::Y, 10.0).arguments.first().unwrap().clone());
        let mut machine = Machine::new(CounterStepper::default());
        let _ = gcode_input.send(gcode);
        machine.poll_task(&gcode_channel);
        machine.step_monitor(1, XYZId::X);
        machine.step_monitor(1, XYZId::Y);
        assert!(!machine.steppers.x.on_target(), "Move requires movement. 1");
        let mut i_x = 0;
        let mut i_y = 0;
        for i in 2..100000 {
            machine.step_monitor(i * 10, XYZId::X);
            machine.step_monitor(i * 10, XYZId::Y);
            if machine.steppers.x.on_target() {
                i_x = i;
            }
            if machine.steppers.y.on_target() {
                i_y = i;
            }
            if machine.steppers.x.on_target() && machine.steppers.y.on_target() {
                break;
            }
        }
        assert_eq!(i_x, i_y, "X and Y should end near each other.");
        assert_ne!(i_x, 0, "X should end at a non zero iteration");
        assert_ne!(i_y, 0, "Y should end at a non zero iteration");
        assert_eq!(machine.steppers.x.get_position(), 1 * RESOLUTION as i32, "XPosition");
        assert_eq!(machine.steppers.y.get_position(), 10 * RESOLUTION as i32, "YPosition");
        assert!(machine.steppers.x.on_target(), "should be on target 10.");
    }

    #[test]
    pub fn machine_specify_feedrate() {
        let gcode_channel = SplitChannel::new(crate::Channel::<GcodeCommand, 3>::default());
        let gcode_input = gcode_channel.create_sender();
        let mut gcode: GcodeCommand = move_command(XYZId::X, 10.0);
        gcode.arguments.push(CommandArgument { mnumonic: ArgumentMnumonic::F, value: MajorMinorNumber { major: 100, minor: 0, float: 100.0 } });
        let mut machine = Machine::new(CounterStepper::default());
        let default_feed_rate = machine.max_feed_rate.clone();
        let _ = gcode_input.send(gcode);
        assert_ne!(default_feed_rate, (100.0 * 60.0 / RES_F32) as u32, "Debug test assert, test feed rate should not be default.");
        machine.poll_task(&gcode_channel);
        machine.step_monitor(1, XYZId::X);

        assert_ne!(machine.steppers.x.get_target(), 0, "Debug test assert. Target needs to be set for feed rate.");
        assert_ne!(machine.feed_rate, default_feed_rate, "Machine feed rate should be changed.");
    }
}
