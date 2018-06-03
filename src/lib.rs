pub mod lazy {
    //use std::rc::Rc;
    use std::cell::RefCell;

    pub struct Thunk<'a, T> {
        _thunk: RefCell<Option<T>>,
        _think: Box<'a + Fn() -> T>,
    }

    impl <'a, T> Thunk<'a, T> {
        pub fn new<F>(clsr: F) -> Thunk<'a, T> where F: 'a + Fn() -> T {
            Thunk {
                _think: Box::new(clsr),
                _thunk: RefCell::new(None),
            }
        }

        pub fn force(self) -> Option<T> {
            if self._thunk.borrow().is_none() {
                let rv = (self._think)();
                self._thunk.replace(Some(rv));
            }
            self._thunk.into_inner()
        }
    }

    //pub struct Cell<T> {
        //pub head: T,
        //tail: Stream<T>,
    //}

    //impl <T> Cell<T> {
        //pub fn new(x: T, tail: Stream<T>) -> Cell<T> {
            //Cell {
                //head: x,
                //tail: tail,
            //}
        //}
    //}

    //pub struct Stream<T> {
        //_cell: Rc<Thunk<Cell<T>>>
    //}

    //impl <T> Stream<T> {
        //// TODO For finite lists.
        ////pub fn empty() {
            ////Stream {
                ////_cell: None
            ////}
        ////}

        //pub fn get(self) -> Option<T> {
            //Rc::try_unwrap(self._cell)
        //}

        //pub fn new(f: Box<Fn() -> Cell<T>>) -> Stream<T> {
            //Stream {
                //_cell: Rc::new(Thunk::new(f))
            //}
        //}
    //}

    //impl <T> Iterator for Stream<T> {
        //type Item = Rc<Thunk<Cell<T>>>;

        //fn next(&mut self) -> Option<Self::Item> {
            //match Rc::try_unwrap(self._cell.clone()) {
                //Ok(thunk) => {
                    //match thunk.force() {
                        //Some(cell) => {
                            //self._cell = cell.tail._cell;
                            //Some(self._cell.clone())
                        //},
                        //None => None
                    //}
                //},
                //_ => None,
            //}
        //}
    //}
}

#[cfg(test)]
mod tests {
    //use lazy::{Thunk, Cell, Stream};
    use lazy::Thunk;
    use std::time::SystemTime;

    #[test]
    fn thunks_defer_application_until_forced() {
        let t = Thunk::new(|| 5);
        let v = t.force();
        assert_eq!(v, Some(5));
    }

    #[test]
    fn thunks_memoize_values() {
        let t = Thunk::new(|| SystemTime::now());
        let p = t.force().unwrap();
        let q = p;
        assert_eq!(p, q);
    }

    //#[test]
    //fn streams_are_lazy_and_infinite() {
        //fn ints_from(n: usize) -> Stream<usize> {
            //Stream::new(Box::new(move || Cell::new(n, ints_from(n+1))))
        //}

        //let mut strm = ints_from(5);
        //let x = strm.next();
        ////assert_eq!(strm.take(5).count(), 5);
        ////for x in strm.iter().take(5) {
            ////assert_eq!(x, x); // Not valid!
        ////}
    //}
}
