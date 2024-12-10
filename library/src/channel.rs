//use arrayvec::ArrayVec;

//trait Sender<T> : Clone
//{
    //fn send(item: T) -> Result<(), T>;
//}

//trait Receiver<T>
//{
    //fn recv(&mut self) -> Option<T>;
//}

//#[derive(Clone)]
//struct MySender<T, F> where F: Fn() -> Result<(), T> {
    //cb: F,
//}

//struct MyRecv<T, const SIZE: usize> {
    //buffer: ArrayVec<T, SIZE>,
//}

//pub fn channel_stack<const SIZE: usize>() -> (impl Sender<T>, impl Receiver<T>) {
    //let recv = MyRecv
//}
