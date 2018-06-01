pub mod lazy {
    pub struct Thunk<T, F: FnOnce() -> T> {
        _think: F,
        _val: Option<T>,
    }

    impl <T, F: FnOnce() -> T> Thunk<T, F> {
        pub fn new(closure: F) -> Thunk<T, F> {
            Thunk {
                _think: closure,
                _val: None
            }
        }

        pub fn force(self) -> T {
            match self._val {
                Some(v) =>
                    v,
                None => {
                    let think = self._think;
                    let rv = think();
                    //self._val = Some(rv);
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
}
