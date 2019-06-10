use crate::range::Size;
use crate::seed::Seed;
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

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
