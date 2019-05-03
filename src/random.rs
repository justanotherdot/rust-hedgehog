use crate::range::Size;
use crate::seed::Seed;

// TODO I've used the F# naming here with the ctor `Random`
// each impl (R, F#, and Haskell) differs in little ways
// between each gen module so I'm trying to find a consistent
// repr. between all three that makes sense to Rusts strengths.
pub type Random<'a, A> = Box<Fn(Seed, Size) -> A + 'a>;

pub fn unsafe_run<'a, A>(seed: Seed, size: Size, r: Random<'a, A>) -> A {
    r(seed, size)
}

pub fn run<'a, A>(seed: Seed, size: Size, r: Random<'a, A>) -> A {
    unsafe_run(seed, size.max(Size(1)), r)
}

pub fn delay<'a, A>(f: Box<Fn() -> Random<'a, A> + 'a>) -> Random<'a, A>
where
    A: 'a,
{
    Box::new(move |seed, size| unsafe_run(seed, size, f()))
}

pub fn map<'a, A, B>(f: Box<Fn(A) -> B>, r: Random<'a, A>) -> Random<'a, B>
where
    A: 'a,
    B: 'a,
{
    Box::new(move |seed, size| f(unsafe_run(seed, size, r)))
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
