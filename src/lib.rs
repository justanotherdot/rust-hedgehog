pub mod lazy {
    #[derive(Copy, Clone)]
    pub struct Thunk<T, F: (FnOnce() -> T) + Copy> {
        _think: F,
        _memo: Option<T>,
    }

    impl<T: Clone + Copy, F: (FnOnce() -> T) + Copy> Thunk<T, F> {
        pub fn new(closure: F) -> Thunk<T, F> {
            Thunk {
                _think: closure,
                _memo: None,
            }
        }

        pub fn force(&mut self) -> T {
            match self._memo {
                Some(v) => v,
                None => {
                    let rv = (self._think)();
                    self._memo = Some(rv.clone());
                    rv
                }
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
        let mut t = Thunk::new(|| 5);
        let v = t.force();
        assert_eq!(v, 5);
    }

    #[test]
    fn it_memoizes_values() {
        let mut t = Thunk::new(|| SystemTime::now() );
        let p = t.force();
        let q = t.force();
        assert_eq!(p, q);
    }
}
