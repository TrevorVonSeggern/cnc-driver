//use core::ops::{Add, Div, Mul, Sub};

//#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
//#[allow(non_camel_case_types)]
//pub struct u32_8fx (pub u32);

//#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
//#[allow(non_camel_case_types)]
//pub struct u64_8fx (pub u64);

//impl u32_8fx {
    //pub fn new(data: u32) -> Self {
        //Self(data << 8)
    //}
    //pub fn from_raw_data(data: u32) -> Self {
        //Self(data)
    //}
//}

//impl From<u32> for u32_8fx {
    //fn from(value: u32) -> Self {
        //Self::new(value)
    //}
//}

//impl From<f32> for u32_8fx {
    //fn from(value: f32) -> Self {
        //const SCALER:f32 = (1 << 8) as f32;
        //Self::from_raw_data((value * SCALER) as u32)
    //}
//}

//impl Add<u32_8fx> for u32_8fx {
    //type Output = u32_8fx;
    //fn add(self, rhs: u32_8fx) -> Self::Output {
        //u32_8fx::from_raw_data(self.0 + rhs.0)
    //}
//}

//impl Sub<u32_8fx> for u32_8fx {
    //type Output = u32_8fx;
    //fn sub(self, rhs: u32_8fx) -> Self::Output {
        //u32_8fx::from_raw_data(self.0 - rhs.0)
    //}
//}

//impl Mul<u32_8fx> for u32_8fx {
    //type Output = u32_8fx;
    //fn mul(self, rhs: u32_8fx) -> Self::Output {
        //u32_8fx::from_raw_data(((self.0 as u64 * rhs.0 as u64)>>8) as u32)
    //}
//}

//impl Div<u32_8fx> for u32_8fx {
    //type Output = u32_8fx;
    //fn div(self, rhs: u32_8fx) -> Self::Output {
        //u32_8fx::from_raw_data((((self.0 as u64) << 8) / rhs.0 as u64) as u32)
    //}
//}


//impl u64_8fx {
    //pub fn new(data: u64) -> Self {
        //Self(data << 8)
    //}
    //pub fn from_raw_data(data: u64) -> Self {
        //Self(data)
    //}
//}

//impl From<u64> for u64_8fx {
    //fn from(value: u64) -> Self {
        //Self::new(value)
    //}
//}

//impl From<f32> for u64_8fx {
    //fn from(value: f32) -> Self {
        //const SCALER:f32 = (1 << 8) as f32;
        //Self::from_raw_data((value * SCALER) as u64)
    //}
//}

//impl From<f64> for u64_8fx {
    //fn from(value: f64) -> Self {
        //const SCALER:f64 = (1 << 8) as f64;
        //Self::from_raw_data((value * SCALER) as u64)
    //}
//}

//impl Add<u64_8fx> for u64_8fx {
    //type Output = u64_8fx;
    //fn add(self, rhs: u64_8fx) -> Self::Output {
        //u64_8fx::from_raw_data(self.0 + rhs.0)
    //}
//}

//impl Add<u32_8fx> for u64_8fx {
    //type Output = u64_8fx;
    //fn add(self, rhs: u32_8fx) -> Self::Output {
        //u64_8fx::from_raw_data(self.0 + rhs.0 as u64)
    //}
//}

//impl Sub<u64_8fx> for u64_8fx {
    //type Output = u64_8fx;
    //fn sub(self, rhs: u64_8fx) -> Self::Output {
        //u64_8fx::from_raw_data(self.0 - rhs.0)
    //}
//}

//#[cfg(test)]
//mod tests {
    //use super::*;

    //#[test]
    //fn from_u32() {
        //let source: u32 = 1;
        //let dest: u32_8fx = u32_8fx::from(source);
        //assert_eq!(dest.0, 0b1_0000_0000u32)
    //}

    //#[test]
    //fn into_u32() {
        //let source: u32 = 1;
        //let dest: u32_8fx = source.into();
        //assert_eq!(dest.0, 0b1_0000_0000u32)
    //}

    //#[test]
    //fn add() {
        //let ten: u32_8fx = 10.into();
        //let two: u32_8fx = 2.into();
        //let dest = ten + two;
        //let twelve: u32_8fx = 12.into();
        //assert_eq!(dest, twelve)
    //}

    //#[test]
    //fn sub() {
        //let ten: u32_8fx = 10.into();
        //let two: u32_8fx = 2.into();
        //let dest = ten - two;
        //assert_eq!(dest, 8.into())
    //}

    //#[test]
    //fn mul() {
        //let ten: u32_8fx = 10.into();
        //let two: u32_8fx = 2.into();
        //let dest = ten * two;
        //assert_eq!(dest, 20.into())
    //}

    //#[test]
    //fn div() {
        //let ten: u32_8fx = 10.into();
        //let two: u32_8fx = 2.into();
        //let dest = ten / two;
        //assert_eq!(dest, 5.into())
    //}
//}
