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

    pub fn map<B, F>(self, f: F) -> Lazy<'a, B>
    where
        F: Fn(A) -> B + 'a,
        A: Clone + 'a,
        B: Clone + 'a,
    {
        Lazy::from_closure(|| f(self.value()))
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

impl<'a, A: Clone + PartialEq> PartialEq for Lazy<'a, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl<'a, A: Clone + Eq> Eq for Lazy<'a, A> {}

#[derive(Clone, Debug)]
pub struct LazyVec<'a, A: Clone>(Lazy<'a, Vec<Lazy<'a, A>>>);

impl<'a, A: Clone + 'a> LazyVec<'a, A> {
    pub fn empty() -> LazyVec<'a, A> {
        LazyVec(Lazy::new(vec![]))
    }

    pub fn singleton(x: A) -> LazyVec<'a, A> {
        LazyVec(Lazy::new(vec![Lazy::new(x)]))
    }

    pub fn from_vec(xs: Vec<Lazy<'a, A>>) -> LazyVec<'a, A> {
        LazyVec(Lazy::new(xs))
    }

    pub fn find<F>(&self, f: &'a F) -> Option<A>
    where
        F: Fn(A) -> bool + 'a,
    {
        self.0
            .value()
            .into_iter()
            .find(|x| f(x.value()))
            .map(|x| x.value())
    }

    pub fn len(&self) -> usize {
        // TODO: This probably ought not to force the value.
        // Perhaps we can clone self and only force the spine by looking at the cloned values len
        // field?
        self.0.value().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn append(self, other: LazyVec<'a, A>) -> LazyVec<'a, A> {
        unimplemented!()
        //self.map(&|xs: Vec<Lazy<'a, A>>| {
        //other.map(&|ys: Vec<Lazy<'a, A>>| -> Vec<Lazy<'a, A>> {
        //xs.into_iter().chain(ys).collect()
        //})
        //})
    }

    pub fn insert(self, pos: usize, elem: A) -> LazyVec<'a, A> {
        LazyVec(self.0.map(move |xs| {
            let mut xs0 = xs.clone();
            xs0.insert(pos, Lazy::new(elem.clone()));
            xs0
        }))
    }

    pub fn push(self, elem: A) -> LazyVec<'a, A> {
        LazyVec(self.0.map(move |xs| {
            let mut xs0 = xs.clone();
            xs0.push(Lazy::new(elem.clone()));
            xs0
        }))
    }

    pub fn map<B, F>(self, f: Rc<F>) -> LazyVec<'a, B>
    where
        F: Fn(A) -> B + 'a,
        A: Clone + 'a,
        B: Clone + 'a,
    {
        LazyVec(
            self.0
                .map(move |xs: Vec<Lazy<'a, A>>| xs.into_iter().map(|x| x.map(f)).collect()),
        )
    }

    pub fn for_each<F>(&self, f: &'a F)
    where
        F: Fn(A) -> () + 'a,
    {
        self.0.value().into_iter().for_each(|x| f(x.value()));
    }

    pub fn fold<B, F>(&self, initial: B, f: &F) -> B
    where
        F: Fn(B, A) -> B + 'a,
        B: Clone + 'a,
    {
        self.0
            .value()
            .into_iter()
            .map(|x| x.value())
            .fold(initial, f)
    }

    pub fn first(&self) -> Option<A> {
        self.get(0)
    }

    pub fn take(self, n: usize) -> LazyVec<'a, A> {
        LazyVec(
            self.0
                .map(move |xs: Vec<Lazy<'a, A>>| xs.into_iter().take(n).collect()),
        )
    }

    pub fn skip(self, n: usize) -> LazyVec<'a, A> {
        LazyVec(
            self.0
                .map(move |xs: Vec<Lazy<'a, A>>| xs.into_iter().skip(n).collect()),
        )
    }

    pub fn get(&self, n: usize) -> Option<A> {
        self.0.value().get(n).map(|x| x.value())
    }

    pub fn flat_map<B, F>(self, f: &'a F) -> LazyVec<'a, B>
    where
        F: Fn(A) -> LazyVec<'a, B> + 'a,
        A: Clone + 'a,
        B: Clone + 'a,
    {
        unimplemented!()
        //LazyVec(
        //self.0
        //.map(&|xs: Vec<Lazy<'a, A>>| xs.into_iter().map(|x| x.map(f)).collect()),
        //)
    }

    pub fn filter<F>(self, f: &'a F) -> LazyVec<'a, A>
    where
        F: Fn(A) -> bool + 'a,
        A: Clone + 'a,
    {
        LazyVec(
            self.0
                .map(move |xs: Vec<Lazy<'a, A>>| xs.into_iter().filter(|x| f(x.value())).collect()),
        )
    }

    pub fn to_vec(&self) -> Vec<Lazy<'a, A>> {
        self.0.value()
    }

    pub fn zip<B>(self, other: LazyVec<'a, B>) -> LazyVec<'a, (A, B)>
    where
        B: Clone + 'a,
    {
        unimplemented!()
        //self.map(&|xs: Vec<Lazy<'a, A>>| {
        //other.map(&|ys: Vec<Lazy<'a, A>>| -> Vec<Lazy<'a, A>> {
        //xs.into_iter().chain(ys).collect()
        //})
        //})
    }

    pub fn all<F>(self, f: &'a F) -> bool
    where
        F: Fn(A) -> bool + 'a,
        A: Clone + 'a,
    {
        self.0
            .map(&|xs: Vec<Lazy<'a, A>>| xs.into_iter().all(|x| f(x.value())))
            .value()
    }
}

macro_rules! lazy_vec(
    [ $( $value:expr ),* ] => {
    {
        let lv = LazyVec::empty();
        $(
            lv.push($value);
        )+
        lv
    }
    }
);

impl<'a, A: Clone + PartialEq> PartialEq for LazyVec<'a, A> {
    fn eq(&self, other: &Self) -> bool {
        self.0.value() == other.0.value()
    }
}

impl<'a, A: Clone + Eq> Eq for LazyVec<'a, A> {}

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
