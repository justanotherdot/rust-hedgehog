use std::cell::RefCell;
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

// TODO: For the moment we use RefCell but if someone tries to force a value on the same node at
// the same time e.g. in parallel, this might cause a panic. This may not actually be a concern.
#[derive(Clone)]
pub struct Lazy<'a, A> {
    value: RefCell<Option<A>>,
    closure: Rc<dyn Fn() -> A + 'a>,
}

impl<'a, A> Lazy<'a, A>
where
    A: Clone + 'a,
{
    pub fn new(value: A) -> Lazy<'a, A> {
        Lazy {
            closure: Rc::new(move || value.clone()),
            value: RefCell::new(None),
        }
    }

    pub fn from_closure<F>(closure: F) -> Lazy<'a, A>
    where
        F: 'a + Fn() -> A,
    {
        Lazy {
            closure: Rc::new(closure),
            value: RefCell::new(None),
        }
    }

    fn force(&self) {
        let mut val = self.value.borrow_mut();
        if val.is_none() {
            let result = (self.closure)();
            *val = Some(result);
        }
    }

    pub fn value(&self) -> A {
        self.force();
        self.value.clone().into_inner().unwrap()
    }

    pub fn map<B, F>(self, f: Rc<F>) -> Lazy<'a, B>
    where
        A: Clone + 'a,
        B: Clone + 'a,
        F: Fn(A) -> B + 'a,
    {
        map(f, self)
    }
}

pub fn map<'a, A, B, F>(f: Rc<F>, l: Lazy<'a, A>) -> Lazy<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B + 'a,
{
    let closure = move || {
        let v = l.value();
        f(v)
    };
    Lazy::from_closure(closure)
}

impl<'a, A> Debug for Lazy<'a, A>
where
    A: Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let val = self.value();
        f.write_str(&format!("{:#?}", val))
    }
}

#[cfg(test)]
mod tests {
    use lazy::Lazy;
    use std::cell::RefCell;
    use std::time::SystemTime;

    #[test]
    fn lazy_defer_application_until_forced() {
        let t = SystemTime::now();
        let l = Lazy::new(t);
        let v = l.value();
        assert_eq!(v, t);
        assert!(v.elapsed().unwrap() != SystemTime::now().elapsed().unwrap());
    }

    #[test]
    fn lazy_memoize_values_01() {
        let n = 42;
        let l = Lazy::new(n);
        l.force();
        l.force();
        assert_eq!(l.value, RefCell::new(Some(n)));
    }

    #[test]
    fn lazy_memoize_values_02() {
        let n = 42;
        let l = Lazy::new(n);
        assert_eq!(l.value(), n);
    }
}
