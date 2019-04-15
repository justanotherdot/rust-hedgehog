pub struct Lazy<'a, T> {
    value: Option<T>,
    closure: Box<'a + Fn() -> T>,
}

impl<'a, T: Clone> Lazy<'a, T> {
    pub fn new<F>(closure: F) -> Lazy<'a, T>
    where
        F: 'a + Fn() -> T,
    {
        Lazy {
            closure: Box::new(closure),
            value: None,
        }
    }

    fn force(&mut self) -> &T {
        if self.value.is_none() {
            self.value = Some((self.closure)());
        }
        &self.value.as_ref().unwrap()
    }

    pub fn value(&mut self) -> &T {
        self.force()
    }
}

#[cfg(test)]
mod tests {
    use lazy::Lazy;
    use std::time::SystemTime;

    #[test]
    fn lazy_defer_application_until_forced() {
        let mut t = Lazy::new(|| SystemTime::now());
        let v = t.value();
        assert!(*v != SystemTime::now());
    }

    #[test]
    fn lazy_memoize_values() {
        let n = 42;
        let mut t = Lazy::new(|| n);
        t.value();
        t.value();
        assert_eq!(*t.value(), n);
    }
}
