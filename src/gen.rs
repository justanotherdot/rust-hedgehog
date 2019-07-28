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

pub fn delay<'a, A>(f: Box<dyn Fn() -> Gen<'a, A> + 'a>) -> Gen<'a, A>
where
    A: Clone + 'a,
{
    let delayed_rnd = random::delay(Rc::new(move || to_random(f())));
    from_random(delayed_rnd)
}

pub fn create<'a, A, F>(shrink: Rc<F>, random: Random<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> Vec<A> + 'a,
{
    let expand = Rc::new(move |x| x);
    let shrink: Rc<F> = shrink.into();
    from_random(random::map(Rc::new(move |x| {
        tree::unfold(expand.clone(), shrink.clone(), x)
    }), random))
}

// TODO: This probably will need to become `apply!` for the primary purpose of doing
pub fn apply<'a, A, B, F>(gf: Gen<'a, F>) -> impl Fn(Gen<'a, A>) -> Gen<'a, B>
where
    F: Fn(A) -> B + Clone + 'a,
    A: Clone + 'a,
    B: Clone + 'a,
{
    move |gx: Gen<A>| {
        let gf = gf.clone();
        bind(gf)(Rc::new(move |f: F| {
            let gx = gx.clone();
            bind(gx)(Rc::new(move |x| constant(f(x))))
        }))
    }
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

// TODO: Turn map into a macro? e.g. map! that is variadic.
pub fn map2<'a, F, A, B, C>(
    f: Rc<F>,
) -> impl Fn(Gen<'a, A>) -> Rc<dyn Fn(Gen<'a, B>) -> Gen<'a, C> + 'a>
where
    A: Clone + 'a,
    B: Clone + 'a,
    C: Clone + 'a,
    F: Fn(A, B) -> C + 'a,
{
    move |gx: Gen<'a, A>| {
        let f = f.clone();
        Rc::new(move |gy: Gen<'a, B>| {
            let f = f.clone();
            let gx = gx.clone();
            bind(gx)(Rc::new(move |x: A| {
                let f = f.clone();
                let gy = gy.clone();
                bind(gy)(Rc::new(move |y: B| {
                    let x = x.clone();
                    let y = y.clone();
                    constant(f(x, y))
                }))
            }))
        })
    }
}

pub fn zip<'a, A, B>(gx: Gen<'a, A>) -> impl Fn(Gen<'a, B>) -> Gen<'a, (A, B)>
where
    A: Clone + 'a,
    B: Clone + 'a,
{
    move |gy: Gen<'a, B>| {
        let gx = gx.clone();
        map2(Rc::new(move |x, y| (x, y)))(gx)(gy)
    }
}

pub fn tuple<'a, A>(g: Gen<'a, A>) -> Gen<'a, (A, A)>
where
    A: Clone + 'a,
{
    let g1 = g.clone();
    let g2 = g;
    zip(g1)(g2)
}

pub fn no_shrink<'a, A>(g: Gen<'a, A>) -> Gen<'a, A>
where
    A: Clone + 'a,
{
    let drop = |t: Tree<'a, A>| Tree::new(tree::outcome(t), vec![]);
    map_tree(Rc::new(drop))(g)
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
        Rc::new(shrink::towards(range::origin(range.clone()))),
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
                            random::constant(Some(tree::filter(p3.clone(), x)))
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

pub fn char<'a>(lo: char) -> impl Fn(char) -> Gen<'a, char> {
    move |hi| {
        // Just pretend we can unwrap for now since that's what other langs do.
        map(Rc::new(move |x| unsafe {
            std::char::from_u32_unchecked(x)
        }))(integral(range::constant(lo as u32, hi as u32)))
    }
}

pub fn unicode_all<'a>() -> Gen<'a, char> {
    char('\0')(std::char::MAX)
}

pub fn digit<'a>() -> Gen<'a, char> {
    char('0')('9')
}

pub fn lower<'a>() -> Gen<'a, char> {
    char('a')('z')
}

pub fn upper<'a>() -> Gen<'a, char> {
    char('A')('Z')
}

pub fn latin1<'a>() -> Gen<'a, char> {
    char('\u{000}')('\u{255}')
}

pub fn unicode<'a>() -> Gen<'a, char> {
    let unicode_all_opt = move |lo, hi| {
        map(Rc::new(move |x| std::char::from_u32(x)))(integral(range::constant(
            lo as u32, hi as u32,
        )))
    };

    let unicode_all_opt_gen = unicode_all_opt('\0', std::char::MAX);
    let reject_none = Rc::new(move |x: Option<char>| x.is_some());

    map(Rc::new(move |x: Option<char>| x.unwrap()))(filter(reject_none)(unicode_all_opt_gen))
}

pub fn alpha<'a>() -> Gen<'a, char> {
    choice(vec![lower(), upper()].into_iter())
}

pub fn alphanum<'a>() -> Gen<'a, char> {
    choice(vec![lower(), upper(), digit()].into_iter())
}

pub fn at_least<'a, A>(n: usize) -> impl Fn(Vec<A>) -> bool
where
    A: Clone + 'a,
{
    move |xs: Vec<A>| n == 0 || !(xs.into_iter().skip(n - 1).collect::<Vec<A>>().is_empty())
}

pub fn vec<'a, A>(range: Range<'a, usize>) -> impl Fn(Gen<'a, A>) -> Gen<'a, Vec<A>>
where
    A: Clone + 'a,
{
    move |g: Gen<'a, A>| {
        let range = range.clone();
        from_random(random::sized(Rc::new(move |size| {
            let g = g.clone();
            let range = range.clone();
            random::bind(random::integral(range.clone()))(Rc::new(move |k| {
                let g = g.clone();
                let range = range.clone();
                let r: Random<'a, Vec<Tree<'a, A>>> = random::replicate(k)(to_random(g.clone()));
                let h = Rc::new(move |r| {
                    let range = range.clone();
                    let r0: Tree<'a, Vec<A>> = shrink::sequence_list(r);
                    let f = Rc::new(move |xs| {
                        let range = range.clone();
                        at_least(range::lower_bound(size, range))(xs)
                    });
                    random::constant(tree::filter(f, r0))
                });
                random::bind(r)(h)
            }))
        })))
    }
}

// TODO: This will actually become posisble when const generics become normalised.
#[allow(dead_code)]
fn array<'a, A>(_range: Range<'a, usize>) -> impl Fn(Gen<'a, A>) -> Gen<'a, [A]>
where
    A: Clone + 'a,
{
    move |_| unimplemented!()
}

/// Feeding this function anything other than `unicode` may result in errors as this checks for
/// valid UTF-8 on construction (per Rust's `String` type).
pub fn string<'a>(range: Range<'a, usize>) -> impl Fn(Gen<'a, char>) -> Gen<'a, String> {
    move |g: Gen<'a, char>| {
        let range = range.clone();
        map(Rc::new(move |cs: Vec<char>| {
            let mut s = String::with_capacity(cs.len());
            cs.into_iter().for_each(|c| s.push(c));
            s
        }))(sized(Rc::new(move |_size| vec(range.clone())(g.clone()))))
    }
}

pub fn bool<'a>() -> Gen<'a, bool> {
    item(vec![false, true].into_iter())
}

// n.b. previously `byte'
pub fn u8<'a>(range: Range<'a, u8>) -> Gen<'a, u8> {
    integral(range)
}

pub fn i8<'a>(range: Range<'a, i8>) -> Gen<'a, i8> {
    integral(range)
}

pub fn u16<'a>(range: Range<'a, u16>) -> Gen<'a, u16> {
    integral(range)
}

pub fn i16<'a>(range: Range<'a, i16>) -> Gen<'a, i16> {
    integral(range)
}

pub fn u32<'a>(range: Range<'a, u32>) -> Gen<'a, u32> {
    integral(range)
}

pub fn i32<'a>(range: Range<'a, i32>) -> Gen<'a, i32> {
    integral(range)
}

pub fn u64<'a>(range: Range<'a, u64>) -> Gen<'a, u64> {
    integral(range)
}

pub fn i64<'a>(range: Range<'a, i64>) -> Gen<'a, i64> {
    integral(range)
}

pub fn usize<'a>(range: Range<'a, usize>) -> Gen<'a, usize> {
    integral(range)
}

pub fn isize<'a>(range: Range<'a, isize>) -> Gen<'a, isize> {
    integral(range)
}

pub fn f64<'a>(range: Range<'a, f64>) -> Gen<'a, f64> {
    let r1 = range.clone();
    create(
        Rc::new(move |x| shrink::towards_float(range::origin(range.clone()))(x)),
        random::f64(r1),
    )
}

pub fn f32<'a>(range: Range<'a, f32>) -> Gen<'a, f32> {
    let r1 = range.clone();
    create(
        Rc::new(move |x| shrink::towards_float(range::origin(range.clone()))(x)),
        random::f32(r1),
    )
}

// TODO:
//   guid
//   datetime

pub fn sample_tree<'a, A>(
    size: Size,
) -> impl Fn(usize) -> Rc<dyn Fn(Gen<'a, A>) -> Vec<Tree<'a, A>>>
where
    A: Clone + 'a,
{
    move |count: usize| {
        Rc::new(move |g: Gen<'a, A>| {
            let seed = seed::random();
            random::run(seed, size, random::replicate(count)(to_random(g)))
        })
    }
}

pub fn sample<'a, A>(size: Size) -> impl Fn(usize) -> Rc<dyn Fn(Gen<'a, A>) -> Vec<A>>
where
    A: Clone + 'a,
{
    move |count: usize| {
        Rc::new(move |g: Gen<'a, A>| {
            sample_tree(size)(count)(g)
                .into_iter()
                .map(move |t| tree::outcome(t))
                .collect()
        })
    }
}

pub fn generate_tree<A>(g: Gen<A>) -> Tree<A>
where
    A: Clone,
{
    let seed = seed::random();
    random::run(seed, Size(30), to_random(g))
}

pub fn print_sample<'a, A>(g: Gen<'a, A>)
where
    A: Clone + Debug + 'a,
{
    let forest = sample_tree(Size(30))(5)(g);
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
        let g = create(shrink::towards(3).into(), rand_fn.clone());
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
        //print_sample(vec(range::constant(1, 3))(u8(range::constant(3, 10))));
        //assert!(false);
    }
}
