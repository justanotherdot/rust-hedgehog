pub mod lazy {
    use std::rc::Rc;
    use std::cell::RefCell;

    pub struct Thunk<'a, T> {
        _thunk: RefCell<Option<T>>,
        _think: Box<'a + Fn() -> T>,
    }

    impl <'a, T: Clone> Thunk<'a, T> {
        pub fn new<F>(clsr: F) -> Thunk<'a, T> where F: 'a + Fn() -> T {
            Thunk {
                _think: Box::new(clsr),
                _thunk: RefCell::new(None),
            }
        }

        pub fn force(&self) -> T {
            if self._thunk.borrow().is_none() {
                let rv = (self._think)();
                self._thunk.replace(Some(rv));
            }
            self._thunk
                .clone()
                .into_inner()
                .unwrap()
        }
    }

    pub struct Cell<'a, T> {
        _v: T,
        _tail: Stream<'a, T>,
    }

    impl <'a, T> Cell<'a, T> {
    }

    pub struct Stream<'a, T> {
        _cell: RefCell<Rc<Thunk<'a, Cell<'a, T>>>>
    }

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
        assert_eq!(v, 5);
    }

    #[test]
    fn thunks_memoize_values() {
        let t = Thunk::new(|| SystemTime::now());
        let p = t.force();
        let q = t.force();
        assert_eq!(p, q);
    }

    //#[test]
    //fn lazy_cells_have_a_head_and_a_tail() {
        //let cell = Cell::new(13, Stream::empty());
        //let x:() = cell.head;
    //}
}
