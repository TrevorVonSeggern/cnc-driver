use array_init::array_init;

#[derive(Clone)]
pub struct CircularBuffer<T, const SIZE: usize> {
    pub data: [T; SIZE],
    pub begin: usize,
    pub length: usize,
}

impl<T, const SIZE: usize> Default for CircularBuffer<T, SIZE> where T: Copy + Default + Sized {
    fn default() -> Self {
        let data = array_init(|_| Default::default());
        Self {
            data,
            length: 0,
            begin: 0,
        }
    }
}

impl<T, const SIZE: usize> CircularBuffer<T, SIZE> {
    fn wrap_index(i: usize) -> usize {
        match i {
            i if i >= SIZE => 0,
            i => i,
        }
    }

    pub fn push(&mut self, data: T) {
        let i = Self::wrap_index(self.begin + self.length);
        self.data[i] = data;
        if self.length == SIZE {
            self.begin = Self::wrap_index(self.begin + 1);
        }
        else {
            self.length += 1;
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn consume(&mut self) -> impl Iterator<Item=T> + use<'_, T, SIZE> where T: Clone {
        let length = self.length;
        let begin = self.begin;
        self.begin = 0;
        self.length = 0;
        (begin..begin+length).into_iter()
            .map(|i| self.data[i % SIZE].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_small() {
        let mut c = CircularBuffer::<u32, 1>::default();
        c.push(123);
        assert_eq!(c.length(), 1);
        assert_eq!(c.data[0], 123);
    }

    #[test]
    fn push_overwrites() {
        let mut c = CircularBuffer::<u32, 1>::default();
        c.push(321);
        assert_eq!(c.length(), 1);
        assert_eq!(c.data[0], 321);
    }

    #[test]
    fn push_begin_end_with_overlap() {
        let mut c = CircularBuffer::<u32, 2>::default();
        assert_eq!(c.begin, 0);
        assert_eq!(c.length, 0);
        c.push(111);
        assert_eq!(c.begin, 0);
        assert_eq!(c.length, 1);
        c.push(222);
        assert_eq!(c.begin, 0);
        assert_eq!(c.length, 2);
        c.push(321);
        assert_eq!(c.begin, 1);
        assert_eq!(c.length, 2);
    }

    #[test]
    fn push_size2_overwrites() {
        let mut c = CircularBuffer::<u32, 2>::default();
        c.push(111);
        c.push(222);
        c.push(321);
        assert_eq!(c.length(), 2);
        assert_eq!(c.data[0], 321);
        assert_eq!(c.data[1], 222);
    }
}

