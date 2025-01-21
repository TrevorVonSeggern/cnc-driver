use arrayvec::ArrayVec;
use library::{u32sqrt, CanRecieve, CommandId, CommandMnumonics, GcodeCommand, StepDir, Stepper, XYZData, XYZId};

use crate::write_uart;

pub enum AbsMode {
    Abs,
    Relative,
}

pub struct Machine<SD: StepDir>
{
    pub steppers: XYZData<Stepper<SD>>,
    motor_max_speed: XYZData<u32>,
    max_feed_rate: u32,
    feed_rate: u32,
    home_offset: XYZData<i32>,
    pub step_resolution: XYZData<f32>,
    command_buffer: ArrayVec<GcodeCommand, 2>,
    abs_mode: AbsMode,
}

static mut ACC_CURVE: [u32; 1000] = [0; 1000];
#[allow(static_mut_refs)]
fn initialize_acc_curve(acc: u32) {
    unsafe{ACC_CURVE[0] = library::first_step_delay::<1_000_000>(acc);}
    let len = unsafe{ACC_CURVE.len()};
    for i in 1..len {
        unsafe{ACC_CURVE[i] = library::inter_step_acc_delay(ACC_CURVE[i-1], i as u32);}
    }
}

#[allow(static_mut_refs)]
impl<SD: StepDir> Machine<SD>
{
    pub fn new(step_dir_fn: SD, stepper_conf: XYZData<f32>, stepper_acc: u32, step_resolution: XYZData<f32>) -> Self {
        initialize_acc_curve(((stepper_acc as f32) * step_resolution.x) as u32);
        let x = Stepper::new(XYZId::X, step_dir_fn.clone(), unsafe{ACC_CURVE.as_ref()});
        let y = Stepper::new(XYZId::Y, step_dir_fn.clone(), unsafe{ACC_CURVE.as_ref()});
        let z = Stepper::new(XYZId::Z, step_dir_fn.clone(), unsafe{ACC_CURVE.as_ref()});
        let speeds = stepper_conf.map(|s| *s as u32);
//(s * steps_per_mm as f32 * MM_TICK_FACTOR) as u32
        Self {
            feed_rate: speeds.x.clone(),
            max_feed_rate: speeds.x.clone(),
            motor_max_speed: speeds,
            steppers: XYZData { x, y, z },
            step_resolution,
            command_buffer: Default::default(),
            home_offset: Default::default(),
            abs_mode: AbsMode::Abs,
        }
    }

    fn move_command(&mut self, mut target: XYZData<Option<i32>>, speed: u32) {
        if target.all(|v| v.is_none()) {
            return;
        }
        let abs_offset = match self.abs_mode {
            AbsMode::Abs => Default::default(),
            AbsMode::Relative => XYZData {
                x: self.steppers.x.get_target(),
                y: self.steppers.y.get_target(),
                z: self.steppers.z.get_target()
            },
        };
        target.x = target.x.map(|x| x + self.home_offset.x + abs_offset.x);
        target.y = target.y.map(|y| y + self.home_offset.y + abs_offset.y);
        target.z = target.z.map(|z| z + self.home_offset.z + abs_offset.z);
        //let mut buffer = str_buf::StrBuf::<100>::new();
        //ufmt::uwriteln!(buffer, "x {} y {} z {}", target.x.unwrap_or_default(), target.y.unwrap_or_default(), target.z.unwrap_or_default()).unwrap();
        //write_uart(buffer.as_str());
        let move_distance = u32sqrt(target.iter().filter_map(|v| *v).map(|v| (v * v) as u32).sum()).min(1);
        for (target, stepper) in target.iter().zip(self.steppers.iter_mut()).filter_map(|(t, s)| t.map(|ss| (ss, s))) {
            if target == stepper.get_target() {
                continue;
            }
            let distance = (target - stepper.get_target()).unsigned_abs();
            let m = distance as f32 / move_distance as f32;
            let next_speed = m * speed as f32;
            stepper.set_target(target, next_speed as u32);
            //stepper.speed = Speed{acceleration: speed.acceleration, speed: speed.speed};
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
                    self.feed_rate = self.feed_argument(&command).unwrap_or(self.feed_rate);
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
                    self.feed_rate = self.feed_argument(&command).unwrap_or(self.feed_rate);
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
            write_uart("no next move. Full stop for motors.\n");

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
