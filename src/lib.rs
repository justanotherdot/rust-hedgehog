pub mod lazy {
    pub struct Thunk<T> {
        _think: Box<Fn() -> T>,
        _memo: Option<T>,
    }

    impl<T: Clone + Copy> Thunk<T> {
        pub fn new(closure: Box<Fn() -> T>) -> Thunk<T> {
            Thunk {
                _think: closure,
                _memo: None,
            }
        }

        pub fn force(&mut self) -> T {
            match self._memo {
                Some(v) => v,
                None => {
                    let think = &self._think;
                    let rv = think();
                    //let rv = (self._think)();
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
