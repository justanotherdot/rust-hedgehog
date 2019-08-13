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
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone)]
pub struct Gen<'a, A, C>(#[allow(dead_code)] Random<'a, Tree<'a, A, C>>)
where
    A: Clone,
    C: Iterator<Item = Tree<'a, A, C>>;

// TODO: Would be handy to have From and Into traits implemented for
// this to avoid a lot of the boilerplate.
pub fn from_random<'a, A, C>(r: Random<'a, Tree<'a, A, C>>) -> Gen<'a, A, C>
where
    A: Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    Gen(r)
}

pub fn to_random<'a, A, C>(g: Gen<'a, A, C>) -> Random<Tree<'a, A, C>>
where
    A: Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    g.0
}

pub fn delay<'a, A, C>(f: Box<dyn Fn() -> Gen<'a, A, C> + 'a>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let delayed_rnd = random::delay(Rc::new(move || to_random(f())));
    from_random(delayed_rnd)
}

pub fn create<'a, A, F, C>(shrink: Rc<F>, random: Random<'a, A>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    F: Fn(A) -> Vec<A> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let expand = Rc::new(move |x| x);
    let shrink: Rc<F> = shrink.into();
    from_random(random::map(
        Rc::new(move |x| tree::unfold(expand.clone(), shrink.clone(), x)),
        random,
    ))
}

// TODO: This probably will need to become `apply!` for the primary purpose of doing
pub fn apply<'a, A, B, F, C, D, E>(gf: Gen<'a, F, E>, gx: Gen<'a, A, C>) -> Gen<'a, B, D>
where
    F: Fn(A) -> B + Clone + 'a,
    A: Clone + 'a,
    B: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
    D: Iterator<Item = Tree<'a, B, D>>,
    E: Iterator<Item = Tree<'a, F, E>>,
{
    bind(
        gf,
        Rc::new(move |f: F| {
            let gx = gx.clone();
            bind(gx, Rc::new(move |x| constant(f(x))))
        }),
    )
}

pub fn constant<'a, A, C>(x: A) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    from_random(random::constant(Tree::singleton(x)))
}

pub fn shrink<'a, F, A, C>(f: Rc<F>, g: Gen<'a, A, C>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    F: Fn(A) -> Vec<A> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    map_tree(Rc::new(move |t| Tree::expand(f.clone(), t)), g)
}

pub fn map_tree<'a, F, A, B, C>(f: Rc<F>, g: Gen<'a, A, C>) -> Gen<'a, B, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(Tree<'a, A, C>) -> Tree<'a, B, C> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    map_random(
        |r: Random<'a, Tree<'a, A, C>>| random::map(f.clone(), r.clone()),
        g,
    )
}

pub fn map_random<'a, F, A, B, C>(f: F, g: Gen<'a, A, C>) -> Gen<'a, B, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(Random<'a, Tree<'a, A, C>>) -> Random<'a, Tree<'a, B, C>>,
{
    from_random(f(to_random(g)))
}

// TODO: Arbitrary levels of maps a la `apply!`?
pub fn map<'a, F, A, B, C>(f: Rc<F>, g: Gen<'a, A, C>) -> Gen<'a, B, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    map_tree(Rc::new(move |x| tree::map(f.clone(), x)), g)
}

// TODO: Turn map into a macro? e.g. map! that is variadic.
pub fn map2<'a, F, A, B, C, I>(f: Rc<F>, gx: Gen<'a, A, I>, gy: Gen<'a, B, I>) -> Gen<'a, C, I>
where
    A: Clone + 'a,
    B: Clone + 'a,
    C: Clone + 'a,
    F: Fn(A, B) -> C + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    bind(
        gx,
        Rc::new(move |x: A| {
            let f = f.clone();
            let gy = gy.clone();
            bind(
                gy,
                Rc::new(move |y: B| {
                    let x = x.clone();
                    let y = y.clone();
                    constant(f(x, y))
                }),
            )
        }),
    )
}

pub fn zip<'a, A, B, C>(gx: Gen<'a, A, C>, gy: Gen<'a, B, C>) -> Gen<'a, (A, B), C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    map2(Rc::new(move |x, y| (x, y)), gx, gy)
}

pub fn tuple<'a, A, C>(g: Gen<'a, A, C>) -> Gen<'a, (A, A), C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let g1 = g.clone();
    let g2 = g;
    zip(g1, g2)
}

pub fn no_shrink<'a, A, C>(g: Gen<'a, A, C>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let drop = |t: Tree<'a, A, C>| Tree::new(tree::outcome(t), vec![]);
    map_tree(Rc::new(drop), g)
}

pub fn sized<'a, F, A, C>(f: Rc<F>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    F: Fn(Size) -> Gen<'a, A, C> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    from_random(random::sized(Rc::new(move |s: Size| to_random(f(s)))))
}

// TODO: Generic num might be useful here.
// I'm simply using isize for starters since that's what Size wraps.
pub fn resize<'a, A, C>(new_size: isize, g: Gen<'a, A, C>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    map_random(
        |r: Random<'a, Tree<'a, A, C>>| random::resize(Size(new_size), r),
        g,
    )
}

pub fn scale<'a, F, A, C>(f: Rc<F>, g: Gen<'a, A, C>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    F: Fn(isize) -> isize + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    sized(Rc::new(move |n: Size| resize(f(n.0), g.clone())))
}

pub fn integral<'a, A, C>(range: Range<'a, A>) -> Gen<'a, A, C>
where
    A: Copy + ToPrimitive + FromPrimitive + Integer + Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let range1 = range.clone();
    create(
        Rc::new(move |x| shrink::towards(range::origin(range1.clone()), x)),
        random::integral(range),
    )
}

fn bind_random<'a, A, B, F, C>(
    m: Random<'a, Tree<'a, A, C>>,
    k: Rc<F>,
) -> Random<'a, Tree<'a, B, C>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Random<'a, Tree<'a, B, C>> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    Rc::new(move |seed0, size| {
        let (seed1, seed2) = seed::split(seed0);
        fn run<'a, X, D>(
            seed: Seed,
            size: Size,
            random: Random<'a, Tree<'a, X, D>>,
        ) -> Tree<'a, X, D>
        where
            X: Clone,
            D: Iterator<Item = Tree<'a, X, D>>,
        {
            random::run(seed.clone(), size.clone(), random.clone())
        }
        let t1 = run(seed1, size, m.clone());
        let k1 = k.clone();
        tree::bind(
            t1,
            Rc::new(move |x: A| run(seed2.clone(), size.clone(), k1(x.clone()))),
        )
    })
}

pub fn bind<'a, A, B, F, C>(m: Gen<'a, A, C>, k: Rc<F>) -> Gen<'a, B, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Gen<'a, B, C> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    from_random(bind_random(
        to_random(m),
        Rc::new(move |x: A| to_random(k(x))),
    ))
}

// n.b. Since we cannot use the term `return` unless it's a macro.
// we instead use the simpler term of `pure`.
pub fn pure<'a, A, C>(a: A) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    constant(a)
}

pub fn item<'a, I, A, C>(xs0: I) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    I: Iterator<Item = A>,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let xs: Vec<A> = xs0.collect();
    if xs.is_empty() {
        panic!("gen::item: 'xs' must have at least one element");
    } else {
        let ix_gen = integral(range::constant(0, xs.len() - 1));
        bind(ix_gen, Rc::new(move |ix: usize| constant(xs[ix].clone())))
    }
}

// TODO: This ought to be an IntoIterator, I believe.
pub fn frequency<'a, I, A, C>(xs0: I) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    I: Iterator<Item = (isize, Gen<'a, A, C>)>,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let xs: Vec<(isize, Gen<'a, A, C>)> = xs0.collect();
    let total = xs.iter().map(|(i, _)| i).sum();

    let pick = move |mut n, ys: Vec<(isize, Gen<'a, A, C>)>| {
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
    bind(n_gen, Rc::new(move |n| pick(n, xs.clone())))
}

pub fn choice<'a, I, A, C>(xs0: I) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    I: Iterator<Item = Gen<'a, A, C>>,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let xs: Vec<Gen<'a, A, C>> = xs0.collect();
    if xs.is_empty() {
        panic!("gen::item: 'xs' must have at least one element");
    } else {
        let ix_gen = integral(range::constant(0, xs.len() - 1));
        bind(ix_gen, Rc::new(move |ix: usize| xs[ix].clone()))
    }
}

pub fn choice_rec<'a, I, A, C>(nonrecs: I, recs: I) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    I: Clone + Iterator<Item = Gen<'a, A, C>> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    sized(Rc::new(move |n| {
        if n <= Size(1) {
            choice(nonrecs.clone())
        } else {
            let halve = |x| x / 2;
            let nonrecs = nonrecs
                .clone()
                .chain(recs.clone().map(|g| scale(Rc::new(halve), g)));
            choice(nonrecs)
        }
    }))
}

fn try_filter_random<'a, A, F, C>(
    p: Rc<F>,
    r0: Random<'a, Tree<'a, A, C>>,
) -> Random<'a, Option<Tree<'a, A, C>>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    fn try_n<'b, B, G, D>(
        p1: Rc<G>,
        r1: Random<'b, Tree<'b, B, D>>,
        k: Size,
        n: Size,
    ) -> Random<'b, Option<Tree<'b, B, D>>>
    where
        B: Clone + 'b,
        G: Fn(B) -> bool + 'b,
        D: Iterator<Item = Tree<'b, B, D>>,
    {
        let p2 = p1.clone();
        if k == Size(0) {
            random::constant(None)
        } else {
            let r0 = r1.clone();
            let r1 = random::resize(Size(2 * k.0 + n.0), r0.clone());
            let r2 = r1.clone();
            let p3 = p2.clone();
            let f = Rc::new(move |x: Tree<'b, B>| {
                if p3(tree::outcome(x.clone())) {
                    random::constant(Some(tree::filter(p3.clone(), x)))
                } else {
                    let size1 = Size(k.0 + 1);
                    try_n(p3.clone(), r1.clone(), size1, Size(n.0 - 1))
                }
            });
            random::bind(r2, f)
        }
    };
    let p1 = p.clone();
    random::sized(Rc::new(move |s: Size| {
        let clamp_size = Size(s.0.max(1));
        try_n(p1.clone(), r0.clone(), Size(0), clamp_size)
    }))
}

pub fn filter<'a, A, F, C>(p: Rc<F>, g: Gen<'a, A, C>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    fn loop0<'b, B, G, D>(p: Rc<G>, g: Gen<'b, B, D>, _: ()) -> Random<'b, Tree<'b, B, D>>
    where
        B: Clone + 'b,
        G: Fn(B) -> bool + 'b,
    {
        let filtered_rand = try_filter_random(p.clone(), to_random(g.clone()));
        let p1 = p.clone();
        let g1 = g.clone();
        let f = Rc::new(move |opt| match opt {
            None => {
                let p2 = p1.clone();
                let g2 = g1.clone();
                let h = Rc::new(move || loop0(p2.clone(), g2.clone(), ()));
                random::sized(Rc::new(move |n: Size| {
                    let size1 = Size(n.0 + 1);
                    let delayed_loop = random::delay(h.clone());
                    random::resize(size1, delayed_loop)
                }))
            }
            Some(x) => random::constant(x),
        });
        random::bind(filtered_rand, f)
    }
    from_random(loop0(p.clone(), g.clone(), ()))
}

pub fn try_filter<'a, A, F, C>(p: Rc<F>, g: Gen<'a, A, C>) -> Gen<'a, Option<A>, C>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    let f = Rc::new(move |x0| match x0 {
        None => random::constant(Tree::singleton(None)),
        Some(x) => random::constant(tree::map(Rc::new(move |v| Some(v)), x)),
    });
    let r = random::bind(try_filter_random(p.clone(), to_random(g)), f);
    from_random(r)
}

pub fn some<'a, A, C>(g: Gen<'a, Option<A>, C>) -> Gen<'a, A, C>
where
    A: Clone + 'a,
{
    let filtered = filter(Rc::new(|x: Option<A>| x.is_some()), g);
    let f = Rc::new(move |x| match x {
        None => panic!("internal error: unexpected None"),
        Some(x) => constant(x),
    });
    bind(filtered, f)
}

pub fn option<'a, A, C>(g: Gen<'a, A, C>) -> Gen<'a, Option<A>, C>
where
    A: Clone + 'a,
{
    let g1 = g.clone();
    sized(Rc::new(move |n: Size| {
        let g2 = g1.clone();
        frequency(
            vec![
                (2, constant(None)),
                (1 + n.0, map(Rc::new(move |x| Some(x)), g2)),
            ]
            .into_iter(),
        )
    }))
}

pub fn char<'a, C>(lo: char, hi: char) -> Gen<'a, char, C> {
    // Just pretend we can unwrap for now since that's what other langs do.
    map(
        Rc::new(move |x| unsafe { std::char::from_u32_unchecked(x) }),
        integral(range::constant(lo as u32, hi as u32)),
    )
}

pub fn unicode_all<'a, C>() -> Gen<'a, char, C> {
    char('\0', std::char::MAX)
}

pub fn digit<'a, C>() -> Gen<'a, char, C> {
    char('0', '9')
}

pub fn lower<'a, C>() -> Gen<'a, char, C> {
    char('a', 'z')
}

pub fn upper<'a, C>() -> Gen<'a, char, C> {
    char('A', 'Z')
}

pub fn latin1<'a, C>() -> Gen<'a, char, C> {
    char('\u{000}', '\u{255}')
}

pub fn unicode<'a, C>() -> Gen<'a, char, C> {
    let unicode_all_opt = move |lo, hi| {
        map(
            Rc::new(move |x| std::char::from_u32(x)),
            integral(range::constant(lo as u32, hi as u32)),
        )
    };

    let unicode_all_opt_gen = unicode_all_opt('\0', std::char::MAX);
    let reject_none = Rc::new(move |x: Option<char>| x.is_some());

    map(
        Rc::new(move |x: Option<char>| x.unwrap()),
        filter(reject_none, unicode_all_opt_gen),
    )
}

pub fn alpha<'a, C>() -> Gen<'a, char, C> {
    choice(vec![lower(), upper()].into_iter())
}

pub fn alphanum<'a, C>() -> Gen<'a, char, C> {
    choice(vec![lower(), upper(), digit()].into_iter())
}

pub fn at_least<'a, A>(n: usize, xs: Vec<A>) -> bool
where
    A: Clone + 'a,
{
    n == 0 || !(xs.into_iter().skip(n - 1).collect::<Vec<A>>().is_empty())
}

pub fn vec<'a, A, C>(range: Range<'a, usize>, g: Gen<'a, A, C>) -> Gen<'a, Vec<A>, C>
where
    A: Clone + 'a,
{
    from_random(random::sized(Rc::new(move |size| {
        let g = g.clone();
        let range = range.clone();
        random::bind(
            random::integral(range.clone()),
            Rc::new(move |k| {
                let g = g.clone();
                let range = range.clone();
                let r: Random<'a, Vec<Tree<'a, A, C>>> = random::replicate(k, to_random(g.clone()));
                let h = Rc::new(move |r| {
                    let range = range.clone();
                    let r0: Tree<'a, Vec<A>> = shrink::sequence_list(r);
                    let f = Rc::new(move |xs| {
                        let range = range.clone();
                        at_least(range::lower_bound(size, range), xs)
                    });
                    random::constant(tree::filter(f, r0))
                });
                random::bind(r, h)
            }),
        )
    })))
}

/// Feeding this function anything other than `unicode` may result in errors as this checks for
/// valid UTF-8 on construction (per Rust's `String` type).
pub fn string<'a, C>(range: Range<'a, usize>, g: Gen<'a, char, C>) -> Gen<'a, String, C> {
    map(
        Rc::new(move |cs: Vec<char>| {
            let mut s = String::with_capacity(cs.len());
            cs.into_iter().for_each(|c| s.push(c));
            s
        }),
        sized(Rc::new(move |_size| vec(range.clone(), g.clone()))),
    )
}

pub fn bool<'a, C>() -> Gen<'a, bool, C> {
    item(vec![false, true].into_iter())
}

// n.b. previously `byte'
pub fn u8<'a, C>(range: Range<'a, u8>) -> Gen<'a, u8, C> {
    integral(range)
}

pub fn i8<'a, C>(range: Range<'a, i8>) -> Gen<'a, i8, C> {
    integral(range)
}

pub fn u16<'a, C>(range: Range<'a, u16>) -> Gen<'a, u16, C> {
    integral(range)
}

pub fn i16<'a, C>(range: Range<'a, i16>) -> Gen<'a, i16, C> {
    integral(range)
}

pub fn u32<'a, C>(range: Range<'a, u32>) -> Gen<'a, u32, C> {
    integral(range)
}

pub fn i32<'a, C>(range: Range<'a, i32>) -> Gen<'a, i32, C> {
    integral(range)
}

pub fn u64<'a, C>(range: Range<'a, u64>) -> Gen<'a, u64, C> {
    integral(range)
}

pub fn i64<'a, C>(range: Range<'a, i64>) -> Gen<'a, i64, C> {
    integral(range)
}

pub fn usize<'a, C>(range: Range<'a, usize>) -> Gen<'a, usize, C> {
    integral(range)
}

pub fn isize<'a, C>(range: Range<'a, isize>) -> Gen<'a, isize, C> {
    integral(range)
}

pub fn f64<'a, C>(range: Range<'a, f64>) -> Gen<'a, f64, C> {
    let r1 = range.clone();
    create(
        Rc::new(move |x| shrink::towards_float(range::origin(range.clone()), x)),
        random::f64(r1),
    )
}

pub fn f32<'a, C>(range: Range<'a, f32>) -> Gen<'a, f32, C> {
    let r1 = range.clone();
    create(
        Rc::new(move |x| shrink::towards_float(range::origin(range.clone()), x)),
        random::f32(r1),
    )
}

// TODO:
//   guid
//   datetime

pub fn sample_tree<'a, A, C>(size: Size, count: usize, g: Gen<'a, A, C>) -> Vec<Tree<'a, A, C>>
where
    A: Clone + 'a,
{
    let seed = seed::random();
    random::run(seed, size, random::replicate(count, to_random(g)))
}

pub fn sample<'a, A, C>(size: Size, count: usize, g: Gen<'a, A, C>) -> Vec<A>
where
    A: Clone + 'a,
{
    sample_tree(size, count, g)
        .into_iter()
        .map(move |t| tree::outcome(t))
        .collect()
}

pub fn generate_tree<A, C>(g: Gen<A, C>) -> Tree<A, C>
where
    A: Clone,
{
    let seed = seed::random();
    random::run(seed, Size(30), to_random(g))
}

pub fn print_sample<'a, A, C>(g: Gen<'a, A, C>)
where
    A: Clone + Debug + 'a,
{
    let forest = sample_tree(Size(30), 5, g);
    forest.into_iter().for_each(|t| {
        println!("=== Outcome ===");
        println!("{:?}", tree::outcome(&t));
        println!("=== Shrinks ===");
        tree::shrinks(t).iter().for_each(|s| {
            println!("{:?}", tree::outcome(s));
        });
        println!(".");
    })
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
        let g = create(Rc::new(move |x| shrink::towards(3, x)), rand_fn.clone());
        let rand_fn1 = to_random(g);
        let global_seed = global();
        assert_eq!(
            Tree::singleton(rand_fn(global_seed.clone(), Size(1))),
            rand_fn1(global_seed, Size(1))
        );
    }

    #[test]
    fn print_sample_works() {
        print_sample(alpha());
        print_sample(i64(range::singleton(12)));
        //print_sample(filter(Rc::new(|x| x == 12), i64(range::singleton(12))));
        //print_sample(vec(range::constant(1, 3))(u8(range::constant(3, 10))));
        //assert!(false);
    }
}
