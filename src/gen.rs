use crate::random;
use crate::random::Random;
use crate::range;
use crate::range::{Range, Size};
use crate::seed;
use crate::seed::Seed;
use crate::shrink;
use crate::tree;
use crate::tree::Tree;
use num::{FromPrimitive, Integer, ToPrimitive};
use std::rc::Rc;

#[derive(Clone)]
pub struct Gen<'a, A>(#[allow(dead_code)] Random<'a, Tree<'a, A>>);

// TODO: Would be handy to have From and Into traits implemented for
// this to avoid a lot of the boilerplate.
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

pub fn sized<'a, F, A>(f: Rc<F>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: Fn(Size) -> Gen<'a, A> + 'a,
{
    from_random(random::sized(Rc::new(move |s: Size| to_random(f(s)))))
}

// TODO: Generic num might be useful here.
// I'm simply using isize for starters since that's what Size wraps.
pub fn resize<'a, A>(new_size: isize) -> impl Fn(Gen<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
{
    move |g: Gen<'a, A>| {
        map_random(|r: Random<'a, Tree<'a, A>>| random::resize(Size(new_size))(r))(g)
    }
}

pub fn scale<'a, F, A>(f: Rc<F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: Fn(isize) -> isize + 'a,
{
    move |g: Gen<'a, A>| {
        // We need this before the interior move below for `sized`.
        let f1 = f.clone();
        sized(Rc::new(move |n: Size| resize(f1(n.0))(g.clone())))
    }
}

pub fn integral<'a, A>(range: Range<'a, A>) -> Gen<'a, A>
where
    A: Copy + ToPrimitive + FromPrimitive + Integer + Clone + 'a,
{
    create(
        Box::new(shrink::towards(range::origin(range.clone()))),
        random::integral(range),
    )
}

// TODO: Needs to accept an Rc'd F.
fn bind_random<'a, A, B, F>(m: Random<'a, Tree<'a, A>>) -> impl Fn(Rc<F>) -> Random<'a, Tree<'a, B>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Random<'a, Tree<'a, B>> + 'a,
{
    move |k: Rc<F>| {
        let m1 = m.clone();
        Rc::new(move |seed0, size| {
            let m2 = m1.clone();
            let k1 = k.clone();
            let (seed1, seed2) = seed::split(seed0);
            fn run<'a, X>(
                seed: Seed,
                size: Size,
            ) -> impl Fn(Random<'a, Tree<'a, X>>) -> Tree<'a, X> {
                move |random| random::run(seed.clone(), size.clone(), random.clone())
            }
            let t1 = run(seed1, size)(m2);
            tree::bind(t1)(Rc::new(move |x: A| {
                run(seed2.clone(), size.clone())(k1(x.clone()))
            }))
        })
    }
}

// TODO: Needs to accept an Rc'd F.
pub fn bind<'a, A, B, F>(m0: Gen<'a, A>) -> impl Fn(F) -> Gen<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Gen<'a, B> + 'a,
{
    let m1 = m0.clone();
    move |k0: F| {
        from_random(bind_random(to_random(m1.clone()))(Rc::new(move |x: A| {
            to_random(k0(x))
        })))
    }
}

// n.b. Since we cannot use the term `return` unless it's a macro.
// we instead use the simpler term of `pure`.
pub fn pure<'a, A>(a: A) -> Gen<'a, A>
where
    A: Clone + 'a,
{
    constant(a)
}

pub fn item<'a, I, A>(xs0: I) -> Gen<'a, A>
where
    A: Clone + 'a,
    I: Iterator<Item = A>,
{
    let xs: Vec<A> = xs0.collect();
    if xs.is_empty() {
        panic!("gen::item: 'xs' must have at least one element");
    } else {
        let ix_gen = integral(range::constant(0, xs.len() - 1));
        bind(ix_gen)(move |ix| constant(xs[ix].clone()))
    }
}

//pub fn frequency<'a, I, A>(xs0: I) -> Gen<'a, A>
//where
//A: Clone + 'a,
//I: Iterator<Item = (isize, Gen<'a, A>)>,
//{
//let xs: Vec<(isize, Gen<'a, A>)> = xs0.collect();
//let total = xs.iter().map(|(i, _)| i).sum();

//if xs.is_empty() {
//panic!("gen::frequencey: 'xs' must have at least one element");
//}

//// TODO: This needs to be a fold.
//// That should solve the undefined var issue below with `selection'.
//let pick = move |n0| {
//let mut n = n0;
//move |ys: Vec<(isize, Gen<'a, A>)>| {
//let mut selection;
//for (k, y) in ys.into_iter() {
//if n <= k {
//selection = y;
//continue;
//} else {
//n = n - k;
//continue;
//}
//}
//return selection;
//}
//};

//let n_gen = integral(range::constant(1, total));
//bind(n_gen)(move |n| pick(n)(xs.clone()))
//}

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
