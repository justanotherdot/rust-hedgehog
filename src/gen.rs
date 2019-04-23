use crate::range::Size;
use crate::seed::Seed;
use crate::tree::Tree;

pub struct Gen<'a, A> {
    #[allow(dead_code)]
    un_gen: Fn(Size, Seed) -> Tree<'a, Option<A>>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
