use crate::range::Size;
use crate::seed::Seed;
use crate::tree::Tree;

pub struct Gen<'a, A> {
    #[allow(dead_code)]
    un_gen: Box<Fn(Size, Seed) -> Tree<'a, Option<A>>>,
}

pub fn run_gen<'a, A>(size: Size, seed: Seed, gen: Gen<'a, A>) -> Tree<'a, Option<A>> {
    let f = gen.un_gen;
    f(size, seed)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
