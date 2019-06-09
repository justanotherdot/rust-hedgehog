//use std::borrow::Cow;
use std::cell::Cell;

pub struct Lazy<'a, A> {
    value: Cell<Option<A>>,
    closure: Box<'a + Fn() -> A>,
}

impl<'a, A: Clone> Lazy<'a, A> {
    pub fn new<F>(closure: F) -> Lazy<'a, A>
    where
        F: 'a + Fn() -> A,
    {
        Lazy {
            closure: Box::new(closure),
            value: Cell::new(None),
        }
    }

    fn force(&self) -> A {
        let v = self.value.take();
        if v.is_some() {
            v.unwrap()
        } else {
            let v = (self.closure)();
            self.value.replace(Some(v));
            let v = self.value.take();
            v.unwrap()
        }
    }

    pub fn value(&self) -> A {
        self.force()
    }
}

#[cfg(test)]
mod tests {
    use lazy::Lazy;
    use std::time::SystemTime;

    #[test]
    fn lazy_defer_application_until_forced() {
        let t = Lazy::new(|| SystemTime::now());
        let v = t.value();
        assert!(v != SystemTime::now());
    }

    #[test]
    fn lazy_memoize_values() {
        let n = 42;
        let t = Lazy::new(|| n);
        t.value();
        t.value();
        assert_eq!(t.value(), n);
    }
}
