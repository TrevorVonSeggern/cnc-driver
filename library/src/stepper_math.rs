// https://www.littlechip.co.nz/blog/a-simple-stepper-motor-control-algorithm
use micromath::F32Ext;

#[allow(unused)]
pub fn u64sqrt(x0: u64) -> u64 {
    let mut x = x0;
    let mut xr = 0; // result register
    let mut q2 = 0x4000_0000_0000_0000u64; // scan-bit register, set to highest possible result bit
    while q2 != 0 {
        if (xr + q2) <= x {
            x -= xr + q2;
            xr >>= 1;
            xr += q2; // test flag
        } else {
            xr >>= 1;
        }
        q2 >>= 2; // shift twice
    }
    // add for rounding, if necessary
    if xr < x { xr + 1 } else { xr }
}

#[allow(unused)]
pub fn u32sqrt(x0: u32) -> u32 {
    let mut x = x0;
    let mut xr = 0; // result register
    let mut q2 = 0x4000_0000u32; // scan-bit register, set to highest possible result bit
    while q2 != 0 {
        if (xr + q2) <= x {
            x -= xr + q2;
            xr >>= 1;
            xr += q2; // test flag
        } else {
            xr >>= 1;
        }
        q2 >>= 2; // shift twice
    }
    // add for rounding, if necessary
    if xr < x { xr + 1 } else { xr }
}

pub fn first_step_delay<const SCALER: u32>(acc: u32) -> u32 {
    //let numerator:u32 = SCALER * 2;
    //(SCALER / 3) * u64sqrt(1000u64 / acc as u64)as u32
    ((SCALER as f32) * (10.0 / acc as f32).sqrt()) as u32
}

//pub fn first_step_delay(acc: f32) -> u32 {
    //(1000.0 * (2.0 * 1.8 / acc).sqrt()) as u32
//}
//pub fn inter_step_acc_delay(previous_delay: u32, step_number: u32) -> u32 {
    //let fourx = (4 * step_number) as f32;
    //(previous_delay as f32 * (fourx - 1.0)/(fourx + 1.0)) as u32
//}

pub fn inter_step_acc_delay(previous_delay: u32, step_number: u32) -> u32 {
    let fourx = 4 * step_number as u32;
    let numerator = fourx - 1;
    let denominator = fourx + 1;
    (previous_delay * numerator)/denominator
}
pub fn inter_step_dec_delay(previous_delay: u32, step_number: u32) -> u32 {
    let fourx = 4 * step_number as u32;
    let numerator = fourx + 1;
    let denominator = fourx - 1;
    (previous_delay * numerator)/denominator
}
//pub fn inter_step_acc_delay(previous_delay: f32, step_number: u32) {
    //let fourx = 4.0 * step_number as f32;
    //previous_delay as f32 * ((fourx - 1)/(fourx + 1))
//}

//#[derive()]
pub struct StepIterator {
    pub target: i32,
    pub position: i32,
    pub direction: i8,
    acc_iteration: u8,
    acc_iteration_stop: u8,
    slew_delay_us: u32,
    acc_table: &'static [u32],
}

impl StepIterator {
    pub fn new(acc_table: &'static [u32]) -> Self {
        Self {
            target: 0,
            position: 0,
            direction: 0,
            acc_iteration: 0,
            acc_iteration_stop: 0,
            slew_delay_us: 0,
            acc_table,
        }
    }

    pub fn set_target(&mut self, target_step: i32, slew_delay_us: u32, stop_slew_us: u32) {
        self.target = target_step;
        let displacement = target_step - self.position;
        self.direction = displacement.clamp(-1, 1) as i8;
        self.slew_delay_us = slew_delay_us;
        self.acc_iteration_stop = if stop_slew_us == 0 { 0 } else { self.acc_table.iter().position(|d| *d <= stop_slew_us).unwrap_or(0) as u8 };
    }
}

impl Iterator for StepIterator {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == self.target {
            return None;
        }
        self.position += self.direction as i32;
        let distance = (self.target - self.position).abs();

        // decelerating
        if distance <= (self.acc_iteration.saturating_sub(self.acc_iteration_stop)) as i32 - 1 {
            self.acc_iteration = self.acc_iteration.saturating_sub(1).clamp(0, self.acc_table.len() as u8 - 1);
            let delay = self.acc_table[self.acc_iteration as usize];
            Some(delay)
        }
        // at speed
        else if self.acc_table[self.acc_iteration as usize] <= self.slew_delay_us {
            Some(self.slew_delay_us)
        }
        // accelerating
        else {
            let delay = self.acc_table[self.acc_iteration as usize];
            self.acc_iteration = self.acc_iteration.saturating_add(1).clamp(0, self.acc_table.len() as u8 - 1);
            Some(delay)
        }
    }
}


#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;
    use super::*;

    #[test]
    fn iter_acc_to_3_and_back() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(6, 1, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(1));
        assert_eq!(step_iter.next(), Some(1));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_only_one_step() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(1, 1, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_only_two_steps() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(2, 1, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_change_target() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(6, 1, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(1));
        step_iter.set_target(7, 1, 0);
        assert_eq!(step_iter.next(), Some(1));
        assert_eq!(step_iter.next(), Some(1));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_max_speed() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(4, 2, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_end_speed() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(4, 2, 2);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_end_speed_way_too_slow() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(4, 2, 2000);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_slower_than_acc() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(2, 2000, 0);
        assert_eq!(step_iter.next(), Some(2000));
        assert_eq!(step_iter.next(), Some(2000));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_negative() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(-2, 2000, 0);
        assert_eq!(step_iter.next(), Some(2000));
        assert_eq!(step_iter.next(), Some(2000));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_negative_then_positive() {
        let mut step_iter = StepIterator::new(&[3, 2, 1]);
        step_iter.set_target(-3, 2, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
        step_iter.set_target(3, 2, 0);
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(2));
        assert_eq!(step_iter.next(), Some(3));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn iter_acc_slew_between_acc() {
        let mut step_iter = StepIterator::new(&[30, 10]);
        step_iter.set_target(3, 20, 0);
        assert_eq!(step_iter.next(), Some(30));
        assert_eq!(step_iter.next(), Some(20));
        assert_eq!(step_iter.next(), Some(30));
        assert_eq!(step_iter.next(), None);
    }

    #[test]
    fn first_delay_not_zero() {
        let asdf = first_step_delay::<1000>(10);
        assert_ne!(asdf, 0);
    }

    #[test]
    fn ten_steps() {
        let first_delay = first_step_delay::<1000>(40);
        let mut data = ArrayVec::<u32, 10>::default();
        data.push(first_delay as u32);
        for i in 1..10 {
            data.push(inter_step_acc_delay(data[i - 1], i as u32));
        }
        //assert_ne!(data.as_slice(), data.as_slice());
    }

    #[test]
    fn acc_dec() {
        let first_delay = first_step_delay::<16000>(100);
        assert_ne!(first_delay, 0);
        let mut acc = ArrayVec::<u32, 10>::default();
        let mut dec = ArrayVec::<u32, 10>::default();
        acc.push(first_delay as u32);
        for i in 1..10 {
            acc.push(inter_step_acc_delay(acc[i - 1], i as u32));
        }
        dec.push(inter_step_dec_delay(acc[9], 10));
        for i in 1..10 {
            dec.push(inter_step_dec_delay(dec[i - 1], 10 - i as u32));
        }
        acc.reverse();
        //assert_eq!(acc.as_slice(), dec.as_slice());
    }

    #[test]
    fn test_with_real_thing() {
        //let acc = 16_000.0 * 2000.0 / 100_000.0;
        let first_delay = first_step_delay::<16_000>(320);
        let mut data = ArrayVec::<u32, 10>::default();
        data.push(first_delay as u32);
        for i in 1..10 {
            data.push(inter_step_acc_delay(data[i - 1], i as u32));
        }
        //assert_ne!(data.as_slice(), data.as_slice());
    }

    //#[test]
    //fn guess_sum() {
        //let acc = 100.0;
        //let first_delay = first_step_delay(acc);
        //const S: usize = 200;
        //let mut data = ArrayVec::<u32, S>::default();
        //data.push(first_delay);
        //for i in 1..S {
            //data.push(inter_step_acc_delay(data[i - 1], i as u32));
        //}
        //let sum:u32 = data.iter().sum();
        //let guess_sum = first_step_delay(acc / (S as f32)) + first_delay;
        //assert_eq!(guess_sum, sum);
    //}
}
