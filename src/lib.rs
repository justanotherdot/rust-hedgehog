pub mod lazy {
    #[derive(Copy, Clone)]
    pub struct Thunk<T: Clone, F: FnOnce() -> T> {
        _think: F,
        _val: Option<T>,
    }

    impl<T: Clone, F: FnOnce() -> T> Thunk<T, F> {
        pub fn new(closure: F) -> Thunk<T, F> {
            Thunk {
                _think: closure,
                _val: None,
            }
        }

        fn think(self) -> T {
            (self._think)()
        }

        pub fn force(mut self) -> T {
            match self._val {
                Some(v) => v,
                None => {
                    let rv = self.think();
                    self._val = Some(rv.clone());
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
    fn it_works() {
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
