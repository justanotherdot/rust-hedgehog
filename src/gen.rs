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
    let delayed_rnd = random::delay(Rc::new(move || to_random(f())));
    from_random(delayed_rnd)
}

pub fn create<'a, A, F>(shrink: Box<F>, random: Random<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: 'a + Fn(A) -> Vec<A>,
{
    let expand = Rc::new(move |x| x);
    let shrink: Rc<F> = shrink.into();
    from_random(random::map(Rc::new(tree::unfold(expand, shrink)), random))
}

pub fn constant<'a, A>(x: A) -> Gen<'a, A>
where
    A: Clone + 'a,
{
    from_random(random::constant(Tree::singleton(x)))
}

pub fn shrink<'a, F, A>(f: Rc<F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> Vec<A> + 'a,
{
    move |g: Gen<'a, A>| map_tree(Tree::expand(f.clone()).into())(g)
}

pub fn map_tree<'a, F, A, B>(f: Rc<F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(Tree<'a, A>) -> Tree<'a, B> + 'a,
{
    move |g: Gen<'a, A>| {
        map_random(|r: Random<'a, Tree<'a, A>>| random::map(f.clone(), r.clone()))(g)
    }
}

pub fn map_random<'a, F, A, B>(f: F) -> impl Fn(Gen<'a, A>) -> Gen<'a, B>
where
    A: Clone + 'a,
    F: Fn(Random<'a, Tree<'a, A>>) -> Random<'a, Tree<'a, B>>,
{
    move |g: Gen<'a, A>| from_random(f(to_random(g)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::range::Size;
    use crate::seed::global;
    use crate::shrink;
    use crate::tree::Tree;

    #[test]
    fn create_works() {
        let rand_fn = Rc::new(|_, _| 3);
        let g = create(shrink::towards(3).into(), rand_fn.clone());
        let rand_fn1 = to_random(g);
        let global_seed = global();
        assert_eq!(
            Tree::singleton(rand_fn(global_seed.clone(), Size(1))),
            rand_fn1(global_seed, Size(1))
        );
    }
}
