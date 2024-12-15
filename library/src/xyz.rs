use core::iter::once;

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
#[derive(Clone)]
pub struct XYZData<T> {
    pub x: T,
    pub y: T,
    pub z: T,
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