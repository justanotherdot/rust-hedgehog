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
pub struct Gen<'a, A>(#[allow(dead_code)] Random<'a, Tree<'a, A>>)
where
    A: Clone;

// TODO: Would be handy to have From and Into traits implemented for
// this to avoid a lot of the boilerplate.
pub fn from_random<'a, A>(r: Random<'a, Tree<'a, A>>) -> Gen<'a, A>
where
    A: Clone,
{
    Gen(r)
}

pub fn to_random<'a, A>(g: Gen<'a, A>) -> Random<Tree<'a, A>>
where
    A: Clone,
{
    g.0
}

pub fn delay<'a, A>(f: Box<Fn() -> Gen<'a, A> + 'a>) -> Gen<'a, A>
where
    A: Clone + 'a,
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
    B: Clone + 'a,
    F: Fn(Random<'a, Tree<'a, A>>) -> Random<'a, Tree<'a, B>>,
{
    move |g: Gen<'a, A>| from_random(f(to_random(g)))
}

pub fn map<'a, F, A, B>(f: Rc<F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B + 'a,
{
    move |g: Gen<'a, A>| map_tree(Rc::new(tree::map(f.clone())))(g)
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
            fn run<'a, X>(seed: Seed, size: Size) -> impl Fn(Random<'a, Tree<'a, X>>) -> Tree<'a, X>
            where
                X: Clone,
            {
                move |random| random::run(seed.clone(), size.clone(), random.clone())
            }
            let t1 = run(seed1, size)(m2);
            tree::bind(t1)(Rc::new(move |x: A| {
                run(seed2.clone(), size.clone())(k1(x.clone()))
            }))
        })
    }
}

pub fn bind<'a, A, B, F>(m0: Gen<'a, A>) -> impl Fn(Rc<F>) -> Gen<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Gen<'a, B> + 'a,
{
    let m1 = m0.clone();
    move |k0: Rc<F>| {
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
        bind(ix_gen)(Rc::new(move |ix: usize| constant(xs[ix].clone())))
    }
}

// TODO: This ought to be an IntoIterator, I believe.
pub fn frequency<'a, I, A>(xs0: I) -> Gen<'a, A>
where
    A: Clone + 'a,
    I: Iterator<Item = (isize, Gen<'a, A>)>,
{
    let xs: Vec<(isize, Gen<'a, A>)> = xs0.collect();
    let total = xs.iter().map(|(i, _)| i).sum();

    let pick = move |mut n, ys: Vec<(isize, Gen<'a, A>)>| {
        ys.into_iter()
            .fold(None, |acc, (k, y)| {
                if n <= k {
                    Some(y)
                } else {
                    n = n - k;
                    acc
                }
            })
            .expect("gen::frequency: 'xs' must have at least one element")
    };

    let n_gen = integral(range::constant(1, total));
    bind(n_gen)(Rc::new(move |n| pick(n, xs.clone())))
}

pub fn choice<'a, I, A>(xs0: I) -> Gen<'a, A>
where
    A: Clone + 'a,
    I: Iterator<Item = Gen<'a, A>>,
{
    let xs: Vec<Gen<'a, A>> = xs0.collect();
    if xs.is_empty() {
        panic!("gen::item: 'xs' must have at least one element");
    } else {
        let ix_gen = integral(range::constant(0, xs.len() - 1));
        bind(ix_gen)(Rc::new(move |ix: usize| xs[ix].clone()))
    }
}

pub fn choice_rec<'a, I, A>(nonrecs: I) -> impl Fn(I) -> Gen<'a, A>
where
    A: Clone + 'a,
    I: Clone + Iterator<Item = Gen<'a, A>> + 'a,
{
    move |recs: I| {
        let nonrecs0 = nonrecs.clone();
        sized(Rc::new(move |n| {
            let nonrecs1 = nonrecs0.clone();
            let recs1 = recs.clone();
            if n <= Size(1) {
                choice(nonrecs1)
            } else {
                let halve = move |x| x / 2;
                let nonrecs = nonrecs1.chain(recs1.map(scale(Rc::new(halve))));
                choice(nonrecs)
            }
        }))
    }
}

fn try_filter_random<'a, A, F>(
    p: Rc<F>,
) -> impl Fn(Random<'a, Tree<'a, A>>) -> Random<'a, Option<Tree<'a, A>>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    move |r0: Random<'a, Tree<'a, A>>| {
        fn try_n<'b, B, G>(
            p1: Rc<G>,
            r1: Random<'b, Tree<'b, B>>,
            k: Size,
        ) -> impl Fn(Size) -> Random<'b, Option<Tree<'b, B>>>
        where
            B: Clone + 'b,
            G: Fn(B) -> bool + 'b,
        {
            let p2 = p1.clone();
            move |n: Size| {
                if k == Size(0) {
                    random::constant(None)
                } else {
                    let r0 = r1.clone();
                    let r1 = random::resize(Size(2 * k.0 + n.0))(r0.clone());
                    let r2 = r1.clone();
                    let p3 = p2.clone();
                    let f = Rc::new(move |x: Tree<'b, B>| {
                        if p3(tree::outcome(x.clone())) {
                            random::constant(Some(tree::filter(p3.clone())(x)))
                        } else {
                            let size1 = Size(k.0 + 1);
                            try_n(p3.clone(), r1.clone(), size1)(Size(n.0 - 1))
                        }
                    });
                    random::bind(r2)(f)
                }
            }
        };
        let p1 = p.clone();
        random::sized(Rc::new(move |s: Size| {
            let clamp_size = Size(s.0.max(1));
            try_n(p1.clone(), r0.clone(), Size(0))(clamp_size)
        }))
    }
}

pub fn filter<'a, A, F>(p: Rc<F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    move |g: Gen<'a, A>| {
        fn loop0<'b, B, G>(p: Rc<G>, g: Gen<'b, B>) -> impl Fn() -> Random<'b, Tree<'b, B>>
        where
            B: Clone + 'b,
            G: Fn(B) -> bool + 'b,
        {
            move || {
                let filtered_rand = try_filter_random(p.clone())(to_random(g.clone()));
                let p1 = p.clone();
                let g1 = g.clone();
                let f = Rc::new(move |opt| match opt {
                    None => {
                        let p2 = p1.clone();
                        let g2 = g1.clone();
                        random::sized(Rc::new(move |n: Size| {
                            let size1 = Size(n.0 + 1);
                            let h = Rc::new(loop0(p2.clone(), g2.clone()));
                            let delayed_loop = random::delay(h);
                            random::resize(size1)(delayed_loop)
                        }))
                    }
                    Some(x) => random::constant(x),
                });
                random::bind(filtered_rand)(f)
            }
        }
        from_random(loop0(p.clone(), g.clone())())
    }
}

pub fn try_filter<'a, A, F>(p: Rc<F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, Option<A>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    move |g| {
        let f = Rc::new(move |x0| match x0 {
            None => random::constant(Tree::singleton(None)),
            Some(x) => random::constant(tree::map(Rc::new(move |v| Some(v)))(x)),
        });
        let r = random::bind(try_filter_random(p.clone())(to_random(g)))(f);
        from_random(r)
    }
}

pub fn some<'a, A>(g: Gen<'a, Option<A>>) -> Gen<'a, A>
where
    A: Clone + 'a,
{
    let filtered = filter(Rc::new(|x: Option<A>| x.is_some()))(g);
    let f = Rc::new(move |x| match x {
        None => panic!("internal error: unexpected None"),
        Some(x) => constant(x),
    });
    bind(filtered)(f)
}

pub fn option<'a, A>(g: Gen<'a, A>) -> Gen<'a, Option<A>>
where
    A: Clone + 'a,
{
    let g1 = g.clone();
    sized(Rc::new(move |n: Size| {
        let g2 = g1.clone();
        frequency(
            vec![
                (2, constant(None)),
                (1 + n.0, map(Rc::new(move |x| Some(x)))(g2)),
            ]
            .into_iter(),
        )
    }))
}

// n.b. missing:
// at_least
// list
// array (vec?)
// seq

pub fn char<'a>(lo: char) -> impl Fn(char) -> Gen<'a, char> {
    move |hi| {
        // Just pretend we can unwrap for now since that's what other langs do.
        map(Rc::new(move |x| std::char::from_u32(x).unwrap()))(integral(range::constant(
            lo as u32, hi as u32,
        )))
    }
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
