use crate::lazy::LazyVec;
use crate::range;
use crate::range::Range;
use crate::range::Size;
use crate::seed;
use crate::seed::Seed;
use num::{FromPrimitive, Integer, ToPrimitive};
use std::rc::Rc;

pub type Random<'a, A> = Rc<dyn Fn(Seed, Size) -> A + 'a>;

pub fn unsafe_run<'a, A>(seed: Seed, size: Size, r: Random<'a, A>) -> A {
    r(seed, size)
}

pub fn run<'a, A>(seed: Seed, size: Size, r: Random<'a, A>) -> A {
    unsafe_run(seed, size.max(Size(1)), r)
}

pub fn delay<'a, A>(f: Rc<dyn Fn() -> Random<'a, A> + 'a>) -> Random<'a, A>
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

pub fn resize<'a, A>(new_size: Size, r: Random<'a, A>) -> Random<'a, A>
where
    A: Clone + 'a,
{
    Rc::new(move |seed, _| run(seed, new_size, r.clone()))
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

pub fn bind<'a, A, B, F>(r0: Random<'a, A>, k: Rc<F>) -> Random<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Random<'a, B> + 'a,
{
    Rc::new(move |seed, size| {
        let seed0 = seed.clone();
        let (_seed1, seed2) = seed::split(seed0);
        unsafe_run(
            seed2,
            size,
            k(unsafe_run(seed.clone(), size.clone(), r0.clone())),
        )
    })
}

pub fn f64(range: Range<f64>) -> Random<f64> {
    Rc::new(move |seed, size| {
        let (lo, hi) = range::bounds(size, range.clone());
        let (x, _) = seed::next_double(lo, hi, seed);
        x
    })
}

pub fn f32(range: Range<f32>) -> Random<f32> {
    Rc::new(move |seed, size| {
        let (lo, hi) = range::bounds(size, range.clone());
        let (x, _) = seed::next_float(lo, hi, seed);
        x
    })
}

pub fn replicate<'a, A>(times: usize, r: Random<'a, A>) -> Random<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
{
    Rc::new(move |seed0: Seed, size: Size| {
        let mut k = times;
        let acc = LazyVec::empty();
        let mut seed = seed0;
        loop {
            if k <= 0 {
                break;
            } else {
                let (seed1, seed2) = seed::split(seed);
                let x = unsafe_run(seed1, size, r.clone());
                let acc = acc.insert(0, x);
                seed = seed2;
                k = k - 1;
                continue;
            }
        }
        acc
    })
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
