pub mod lazy {
    #[derive(Copy, Clone)]
    pub struct Thunk<T: Clone, F: FnOnce() -> T> {
        _think: F,
        _memo: Option<T>,
    }

    impl<T: Clone, F: FnOnce() -> T> Thunk<T, F> {
        pub fn new(closure: F) -> Thunk<T, F> {
            Thunk {
                _think: closure,
                _memo: None,
            }
        }

        pub fn force(mut self) -> T {
            match self._memo {
                Some(v) => v,
                None => {
                    let think = self._think;
                    let rv = think();
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

    #[test]
    fn it_defers_application_until_forced() {
        let t = Thunk::new(|| 5);
        let v = t.force();
        assert_eq!(v, 5);
    }

    #[test]
    fn it_memoizes_values() {
        let t = Thunk::new(|| 5);
        t.force();
        let v = t.force();
        assert_eq!(v, 5);
    }
}
