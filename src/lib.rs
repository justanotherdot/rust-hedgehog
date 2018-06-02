pub mod lazy {
    use std::rc::{Rc};

    pub struct Thunk<T> {
        _memo: Option<T>,
        _think: Box<Fn() -> T>,
    }

    impl <T> Thunk<T> {
        pub fn new(clsr: Box<Fn() -> T>) -> Thunk<T> {
            Thunk {
                _think: clsr,
                _memo: None,
            }
        }

        pub fn force(&mut self) -> &Option<T> {
            match self._memo {
                None => {
                    let rv = (self._think)();
                    self._memo = Some(rv);
                }
                _ => ()
            }
            &self._memo
        }
    }

    pub struct Cell<T> {
        _v: T,
        _tail: Stream<T>,
    }

    impl <T> Cell<T> {
        pub fn new(x: T, tail: Stream<T>) -> Cell<T> {
            Cell {
                _v: x,
                _tail: tail,
            }
        }

        pub fn head(self) -> T {
            self._v
        }

        pub fn tail(self) -> Stream<T> {
            self._tail
        }
    }

    pub struct Stream<T> {
        _cell: Rc<Thunk<Cell<T>>>
    }

    impl <T> Stream<T> {
        pub fn new(self, f: Box<Fn() -> Cell<T>>) -> Stream<T> {
            Stream {
                _cell: Rc::new(Thunk::new(f))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lazy::Thunk;
    use std::time::SystemTime;

    #[test]
    fn it_defers_application_until_forced() {
        let mut t = Thunk::new(Box::new(|| 5));
        let v = t.force().unwrap();
        assert_eq!(v, 5);
    }

    #[test]
    fn it_memoizes_values() {
        let mut t = Thunk::new(Box::new(|| SystemTime::now()));
        let p = t.force().unwrap();
        let q = t.force().unwrap();
        assert_eq!(p, q);
    }
}
