use core::{iter::once, ops::{Add, Mul, Sub}};

use crate::ArgumentMnumonic;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum XYZId { X, Y, Z }
pub static XYZ_ID_LIST:[XYZId;3] = [XYZId::X, XYZId::Y, XYZId::Z];

#[derive(Clone)]
pub enum XYZOne<T> {
    X(T),
    Y(T),
    Z(T),
}
#[derive(Clone, Copy)]
pub struct XYZData<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Add for XYZData<T> where T: Add::<Output=T> {
    type Output=XYZData<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> Add<XYZData<T>> for XYZData<Option<T>> where T: Add::<Output=T> {
    type Output=XYZData<Option<T>>;
    fn add(self, rhs: XYZData<T>) -> Self::Output {
        Self {
            x: self.x.map(|x| x + rhs.x),
            y: self.y.map(|y| y + rhs.y),
            z: self.z.map(|z| z + rhs.z),
        }
    }
}

impl<T> Sub for XYZData<T> where T: Sub::<Output=T> {
    type Output=XYZData<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> Sub<XYZData<T>> for XYZData<Option<T>> where T: Sub::<Output=T> {
    type Output=XYZData<Option<T>>;
    fn sub(self, rhs: XYZData<T>) -> Self::Output {
        Self {
            x: self.x.map(|x| x - rhs.x),
            y: self.y.map(|y| y - rhs.y),
            z: self.z.map(|z| z - rhs.z),
        }
    }
}


impl<T> Mul for XYZData<T> where T: Mul::<Output=T> {
    type Output=XYZData<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T> Default for XYZData<T> where T: Default {
    fn default() -> Self {
        return Self {
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
        }
    }
}

impl<T> XYZData<T> {
    pub fn from_clone(state: T) -> Self where T: Clone {
        Self {
            x: state.clone(),
            y: state.clone(),
            z: state,
        }
    }
    pub fn as_ref_array(&self) -> [&T;3] { [&self.x, &self.y, &self.z] }
    pub fn as_ref_array_mut(&mut self) -> [&mut T;3] { [&mut self.x, &mut self.y, &mut self.z] }
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        once(&self.x).chain(once(&self.y)).chain(once(&self.z))
            //.zip([XYZId::X, XYZId::Y, XYZId::Z])
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> {
        once(&mut self.x).chain(once(&mut self.y)).chain(once(&mut self.z))
    }
    pub fn match_id(&self, id: XYZId) -> &T {
        match id {
            XYZId::X => &self.x,
            XYZId::Y => &self.y,
            XYZId::Z => &self.z,
        }
    }
    pub fn match_id_mut(&mut self, id: XYZId) -> &mut T {
        match id {
            XYZId::X => &mut self.x,
            XYZId::Y => &mut self.y,
            XYZId::Z => &mut self.z,
        }
    }

    pub fn all(&self, predicate: impl Fn(&T) -> bool) -> bool {
        predicate(&self.x) && predicate(&self.y) && predicate(&self.z)
    }

    pub fn one_map_mut<TR>(&mut self, axis: XYZId, action: impl Fn(&mut T) -> TR) -> TR {
        match axis {
            XYZId::X => action(&mut self.x),
            XYZId::Y => action(&mut self.y),
            XYZId::Z => action(&mut self.z),
        }
    }

    pub fn map<TR>(&self, p: impl Fn(&T) -> TR) -> XYZData<TR> {
        XYZData { x: p(&self.x), y: p(&self.y), z: p(&self.z) }
    }
}

impl XYZId {
    pub fn from_arg(value: ArgumentMnumonic) -> Option<Self> {
        match value {
            ArgumentMnumonic::X => Some(XYZId::X),
            ArgumentMnumonic::Y => Some(XYZId::Y),
            ArgumentMnumonic::Z => Some(XYZId::Z),
            _ => None,
        }
    }
}
