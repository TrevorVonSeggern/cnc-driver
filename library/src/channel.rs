use core::{cell::UnsafeCell, marker::PhantomData};

use arrayvec::ArrayVec;

#[derive(Default)]
pub struct Channel<T, const SIZE: usize> {
    buffer: ArrayVec<T, SIZE>,
}

fn send<T, const SIZE: usize>(buffer: &mut ArrayVec<T, SIZE>, item: T) -> Result<(), T> {
    if buffer.remaining_capacity() != 0 {
        buffer.push(item);
        Ok(())
    }
    else { Err(item) }
}
fn recieve<T, const SIZE: usize>(buffer: &mut ArrayVec<T, SIZE>,) -> Option<T> {
    if buffer.len() == 0 { None }
    else { buffer.drain(0..1).next() }
}

pub trait CanSendMut<T> {
    fn send_mut(&mut self, item: T) -> Result<(), T>;
}
pub trait CanSend<T> {
    fn send(&self, item: T) -> Result<(), T>;
}
pub trait CanRecieveMut<T> {
    fn recieve_mut(&mut self) -> Option<T>;
}
pub trait CanRecieve<T> {
    fn recieve(&self) -> Option<T>;
}
pub trait CanCreateSenders<T> {
    fn create_sender(&self) -> impl CanSend<T>;
}

impl<T, const SIZE: usize> CanSendMut<T> for Channel<T, SIZE> {
    fn send_mut(&mut self, item: T) -> Result<(), T> { send(&mut self.buffer, item) }
}
impl<T, const SIZE: usize> CanRecieveMut<T> for Channel<T, SIZE> {
    fn recieve_mut(&mut self) -> Option<T> { recieve(&mut self.buffer) }
}

pub struct SplitChannel<T, C> where C: CanRecieveMut<T> + CanSendMut<T> {
    interior: UnsafeCell<C>,
    phantom: PhantomData<T>,
}
impl<T, C> SplitChannel<T, C> where C: CanRecieveMut<T> + CanSendMut<T> {
    pub fn new(channel: C) -> Self {
        Self { interior: UnsafeCell::new(channel), phantom: PhantomData::default() }
    }
}
impl<T, C> CanSend<T> for SplitChannel<T, C> where C: CanRecieveMut<T> + CanSendMut<T> {
    fn send(&self, item: T) -> Result<(), T> {
        unsafe{&mut *self.interior.get()}.send_mut(item)
    }
}
impl<T, C> CanRecieve<T> for SplitChannel<T, C> where C: CanRecieveMut<T> + CanSendMut<T> {
    fn recieve(&self) -> Option<T> {
        unsafe{&mut *self.interior.get()}.recieve_mut()
    }
}
impl<T, C> CanCreateSenders<T> for SplitChannel<T, C> where C: CanRecieveMut<T> + CanSendMut<T> {
    fn create_sender(&self) -> impl CanSend<T> {
        SenderSplit::new(self)
    }
}

pub struct SenderSplit<'a, T, C> where C: CanSendMut<T> + CanRecieveMut<T> {
    interior: &'a SplitChannel<T, C>,
    phantom: PhantomData<T>,
}
impl<'a, T, C> SenderSplit<'a, T, C> where C: CanSendMut<T> + CanRecieveMut<T> {
    fn new(channel: &'a SplitChannel<T, C>) -> Self {
        Self { interior: channel, phantom: PhantomData::default() }
    }
}
impl<'a, T, C> CanSend<T> for SenderSplit<'a, T, C> where C: CanSendMut<T> + CanRecieveMut<T> {
    fn send(&self, item: T) -> Result<(), T> {
        self.interior.send(item)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_send_recieve() {
        let mut c = Channel::<u32, 1>::default();
        assert_eq!(c.send_mut(123), Ok(()));
        assert_eq!(c.recieve_mut(), Some(123))
    }

    #[test]
    fn channel_cannot_push_exceed() {
        let mut c = Channel::<u32, 1>::default();
        assert_eq!(c.send_mut(123), Ok(()));
        assert_eq!(c.send_mut(123), Err(123));
    }

    #[test]
    fn channel_cannot_push_exceed_size2() {
        let mut c = Channel::<u32, 2>::default();
        assert_eq!(c.send_mut(123), Ok(()));
        assert_eq!(c.send_mut(123), Ok(()));
        assert_eq!(c.send_mut(123), Err(123));
    }

    #[test]
    fn channel_cannot_take_none() {
        let mut c = Channel::<u32, 1>::default();
        assert_eq!(c.recieve_mut(), None);
    }

    #[test]
    fn channel_cannot_take_none_size2() {
        let mut c = Channel::<u32, 2>::default();
        assert_eq!(c.send_mut(123), Ok(()));
        assert_eq!(c.recieve_mut(), Some(123));
        assert_eq!(c.recieve_mut(), None);
    }

    #[test]
    fn split_channel_send_revieve() {
        let c = SplitChannel::new(Channel::<u32, 2>::default());
        let sender = c.create_sender();
        assert_eq!(sender.send(123), Ok(()));
        assert_eq!(c.recieve(), Some(123));
        assert_eq!(c.recieve(), None);
    }
}
