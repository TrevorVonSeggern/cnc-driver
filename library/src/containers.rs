//use arrayvec::ArrayVec;

//pub trait Container<T> {
    //fn push(&mut self, item: T);
    //fn get(&self, index: usize) -> Option<&T>;
    //fn get_mut(&mut self, index: usize) -> Option<&mut T>;
    //fn remove_at(&mut self, index: usize);
//}

//pub struct ContainerArrayVecWrapper<T, const SIZE: usize> (ArrayVec<T, SIZE>);

//impl<T, const SIZE: usize> Container<T> for ContainerArrayVecWrapper<T, SIZE> {
    //fn push(&mut self, item: T) {
        //self.0.push(item);
    //}

    //fn remove_at(&mut self, index: usize) {
        //self.0.remove(index);
    //}

    //fn get(&self, index: usize) -> Option<&T> {
        //self.0.get(index)
    //}

    //fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        //self.0.get_mut(index)
    //}
//}
