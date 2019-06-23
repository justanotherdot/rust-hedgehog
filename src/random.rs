use crate::range;
use crate::range::Range;
use crate::range::Size;
use crate::seed;
use crate::seed::Seed;
use num::{FromPrimitive, Integer, ToPrimitive};
use std::rc::Rc;

// TODO I've used the F# naming here with the ctor `Random`
// each impl (R, F#, and Haskell) differs in little ways
// between each gen module so I'm trying to find a consistent
// repr. between all three that makes sense to Rusts strengths.
// TODO: Might make sense to have this as a Lazy.
pub type Random<'a, A> = Rc<Fn(Seed, Size) -> A + 'a>;

pub fn unsafe_run<'a, A>(seed: Seed, size: Size, r: Random<'a, A>) -> A {
    r(seed, size)
}

pub fn run<'a, A>(seed: Seed, size: Size, r: Random<'a, A>) -> A {
    unsafe_run(seed, size.max(Size(1)), r)
}

pub fn delay<'a, A>(f: Rc<Fn() -> Random<'a, A> + 'a>) -> Random<'a, A>
where
    A: 'a,
{
    // TODO: This ought to probably use the Lazy struct.
    Rc::new(move |seed, size| unsafe_run(seed, size, f()))
}

pub fn map<'a, A, B, F>(f: Rc<F>, r: Random<'a, A>) -> Random<'a, B>
where
    A: 'a,
    B: 'a,
    F: 'a + Fn(A) -> B,
{
    Rc::new(move |seed, size| f(unsafe_run(seed, size, r.clone())))
}

pub fn constant<'a, A>(x: A) -> Random<'a, A>
where
    A: Clone + 'a,
{
    Rc::new(move |_, _| x.clone())
}

pub fn sized<'a, F, A>(f: Rc<F>) -> Random<'a, A>
where
    A: Clone + 'a,
    F: Fn(Size) -> Random<'a, A> + 'a,
{
    Rc::new(move |seed, size| unsafe_run(seed, size, f(size)))
}

pub fn resize<'a, A>(new_size: Size) -> impl Fn(Random<'a, A>) -> Random<'a, A>
where
    A: Clone + 'a,
{
    move |r: Random<'a, A>| Rc::new(move |seed, _| run(seed, new_size, r.clone()))
}

pub fn integral<'a, A>(range: Range<'a, A>) -> Random<'a, A>
where
    A: Copy + ToPrimitive + FromPrimitive + Integer,
{
    Rc::new(move |seed, size| {
        let (lo, hi) = range::bounds(size, range.clone());
        let (x, _) = seed::next_integer(lo.to_isize().unwrap(), hi.to_isize().unwrap(), seed);
        FromPrimitive::from_isize(x).unwrap()
    })
}

pub fn bind<'a, A, B, F>(r0: Random<'a, A>) -> impl Fn(Rc<F>) -> Random<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Random<'a, B> + 'a,
{
    let r1 = r0.clone();
    move |k: Rc<F>| {
        Rc::new(move |seed, size| {
            let (seed1, seed2) = seed::split(seed);
            unsafe_run(seed2, size, k(unsafe_run(seed, size, r1)))
        })
    }
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
