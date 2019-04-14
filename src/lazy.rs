pub struct Thunk<'a, T> {
    value: Option<T>,
    closure: Box<'a + Fn() -> T>,
}

impl<'a, T: Clone> Thunk<'a, T> {
    pub fn new<F>(closure: F) -> Thunk<'a, T>
    where
        F: 'a + Fn() -> T,
    {
        Thunk {
            closure: Box::new(closure),
            value: None,
        }
    }

    pub fn force(&mut self) -> &Self {
        if self.value.is_none() {
            self.value = Some((self.closure)());
        }
        self
    }

    pub fn value(&self) -> &T {
        &self.value.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use lazy::Thunk;
    use std::time::SystemTime;

    #[test]
    fn thunks_defer_application_until_forced() {
        let mut t = Thunk::new(|| SystemTime::now());
        let v = t.force().value();
        assert!(*v != SystemTime::now());
    }

    #[test]
    fn thunks_memoize_values() {
        let n = 42;
        let mut t = Thunk::new(|| n);
        t.force();
        t.force();
        assert_eq!(*t.value(), n);
    }
}
