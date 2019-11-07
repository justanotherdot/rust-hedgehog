use crate::range;
use crate::range::Range;
use crate::range::Size;
use crate::seed;
use crate::seed::Seed;
use num::{FromPrimitive, Integer, ToPrimitive};
use std::rc::Rc;

pub type Random<'a, A> = Rc<dyn Fn(&Seed, &Size) -> A + 'a>;

pub fn unsafe_run<'a, A>(seed: &Seed, size: &Size, r: &Random<'a, A>) -> A {
    r(seed, size)
}

pub fn run<'a, A>(seed: &Seed, size: &Size, r: &Random<'a, A>) -> A {
    unsafe_run(seed, size.max(&Size(1)), r)
}

pub fn delay<'a, A: 'a, F>(f: Rc<F>) -> Random<'a, A>
where
    F: Fn() -> Random<'a, A> + 'a,
{
    Rc::new(move |seed, size| unsafe_run(seed, size, &f()))
}

pub fn map<'a, A, B, F>(f: Rc<F>, r: &'a Random<'a, A>) -> Random<'a, B>
where
    A: 'a,
    B: 'a,
    F: Fn(&A) -> B + 'a,
{
    Rc::new(|seed, size| f(&unsafe_run(seed, size, r)))
}

pub fn constant<'a, A: 'a>(x: A) -> Random<'a, A> {
    Rc::new(move |_, _| x)
}

pub fn sized<'a, F, A>(f: Rc<F>) -> Random<'a, A>
where
    F: Fn(&Size) -> Random<'a, A> + 'a,
{
    Rc::new(move |seed, size| unsafe_run(seed, size, &f(size)))
}

pub fn resize<'a, A: 'a>(new_size: &'a Size, r: &'a Random<'a, A>) -> Random<'a, A> {
    Rc::new(|seed, _| run(seed, new_size, r))
}

pub fn integral<'a, A>(range: Range<'a, A>) -> Random<'a, A>
where
    A: Copy + ToPrimitive + FromPrimitive + Integer,
{
    Rc::new(|seed, size| {
        let (lo, hi) = range::bounds(size, range);
        let (x, _) = seed::next_integer(lo.to_isize().unwrap(), hi.to_isize().unwrap(), seed);
        FromPrimitive::from_isize(x).unwrap()
    })
}

pub fn bind<'a, A: 'a, B, F>(r0: &'a Random<'a, A>, k: Rc<F>) -> Random<'a, B>
where
    F: Fn(&A) -> Random<'a, B> + 'a,
{
    Rc::new(|seed, size| {
        let seed0 = seed.clone();
        let (_seed1, seed2) = seed::split(seed0);
        unsafe_run(&seed2, &size, &k(&unsafe_run(seed, size, r0)))
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

pub fn replicate<'a, A: 'a>(times: usize, r: Random<'a, A>) -> Random<'a, Vec<A>> {
    Rc::new(move |seed0: Seed, size: Size| {
        let mut k = times;
        let mut acc = vec![];
        let mut seed = seed0;

        loop {
            if k <= 0 {
                break;
            } else {
                let (seed1, seed2) = seed::split(seed);
                let x = unsafe_run(seed1, size, r.clone());
                acc.insert(0, x);

                seed = seed2;
                k = k - 1;
                continue;
            }
        }

        acc
    })
}
