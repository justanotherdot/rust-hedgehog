use crate::random;
use crate::random::Random;
use crate::tree;
use crate::tree::Tree;
use std::rc::Rc;

pub struct Gen<'a, A>(#[allow(dead_code)] Random<'a, Tree<'a, A>>);

pub fn from_random<'a, A>(r: Random<'a, Tree<'a, A>>) -> Gen<'a, A> {
    Gen(r)
}

pub fn to_random<'a, A>(g: Gen<'a, A>) -> Random<Tree<'a, A>> {
    g.0
}

pub fn delay<'a, A>(f: Box<Fn() -> Gen<'a, A> + 'a>) -> Gen<'a, A>
where
    A: 'a,
{
    let delayed_rnd = random::delay(Box::new(move || to_random(f())));
    from_random(delayed_rnd)
}

pub fn create<'a, A, F>(shrink: Rc<F>, random: Random<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> &'a [A],
{
    let expand = Rc::new(move |x| x);
    from_random(random::map(tree::unfold(expand, shrink), random))
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub_for_gen() {
        assert_eq!(1 + 1, 2);
    }
}
