// https://www.littlechip.co.nz/blog/a-simple-stepper-motor-control-algorithm
use micromath::F32Ext;

#[allow(unused)]
fn u64sqrt(x0: u64) -> u64 {
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
fn u32sqrt(x0: u32) -> u32 {
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


#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;
    use super::*;

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
        let acc = 16_000.0 * 2000.0 / 100_000.0;
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
