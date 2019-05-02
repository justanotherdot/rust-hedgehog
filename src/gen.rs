use crate::range::Size;
use crate::seed::Seed;
use crate::tree::Tree;

pub struct Gen<'a, A>(#[allow(dead_code)] Box<Fn(Size, Seed) -> Tree<'a, A>>);

// TODO I've used the F# naming here with the ctor `Random`
// each impl (R, F#, and Haskell) differs in little ways
// between each gen module so I'm trying to find a consistent
// repr. between all three that makes sense to Rusts strengths.
type Random<'a, A> = Box<Fn(Size, Seed) -> A>;

pub fn from_random<'a, A>(r: Random<Tree<'a, A>>) -> Gen<'a, A> {
    Gen(r)
}

pub fn to_random<'a, A>(g: Gen<'a, A>) -> Random<Tree<'a, A>> {
    g.0
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
