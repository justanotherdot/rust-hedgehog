extern crate num;

use self::num::{Float, FromPrimitive, Integer};
use crate::lazy::{Lazy, LazyVec};
use crate::tree;
use crate::tree::Tree;
use std::rc::Rc;

// This probably could be optimised for an eager language. by simply manipulating the vector
// directly and doing the inner check, rather than returning the function here for use in a
// pipeline a la the F# port.
fn cons_nub<'a, A: 'a>(x: A) -> Box<dyn Fn(LazyVec<'a, A>) -> LazyVec<'a, A> + 'a>
where
    A: Integer + FromPrimitive + Copy,
{
    let cons_nub_do = move |ys0: LazyVec<'a, A>| match ys0.first() {
        None => LazyVec::empty(),
        Some(y) if x == y => ys0,
        Some(_) => ys0.map(|ys2| {
            ys2.insert(0, x);
            ys2
        }),
    };
    Box::new(cons_nub_do)
}

fn unfold<'a, A, B, F>(f: F, b0: B) -> LazyVec<'a, A>
where
    A: Clone + 'a,
    F: Fn(B) -> Option<(A, B)>,
{
    let mut acc = vec![];
    let mut b = b0;
    loop {
        if let Some((a, b1)) = f(b) {
            acc.push(Lazy::new(a));
            b = b1;
            continue;
        } else {
            break;
        }
    }
    LazyVec::from_vec(acc)
}

pub fn removes<'a, A, B>(k0: B, xs0: LazyVec<'a, A>) -> LazyVec<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
    B: Integer + FromPrimitive + Copy + 'a,
{
    fn loop0<'b, C, D>(k: C, n: C, xs: LazyVec<'b, D>) -> LazyVec<'b, LazyVec<'b, D>>
    where
        C: Integer + FromPrimitive + Copy,
        D: Clone,
    {
        let hd = &xs.clone().take(1).get(0).unwrap();
        let tl: LazyVec<_> = xs.clone().skip(1);
        if k > n {
            LazyVec::empty()
        } else if tl.is_empty() {
            LazyVec::singleton(LazyVec::empty())
        } else {
            let inner: LazyVec<_> = loop0(k, n - k, tl.clone()).map(move |mut x| {
                let hd = hd.clone();
                x.push(hd);
                x
            });
            let inner = inner.insert(0, tl);
            inner
        }
    }
    let gen_len = FromPrimitive::from_usize(xs0.len()).unwrap();
    loop0(k0, gen_len, xs0)
}

pub fn elems<'a, A, F>(shrink: Rc<F>, xs00: LazyVec<'a, A>) -> LazyVec<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
    F: Fn(A) -> LazyVec<'a, A> + 'a,
{
    if xs00.is_empty() {
        LazyVec::empty()
    } else {
        let xs01 = xs00.clone().take(1);
        let x0 = xs01.get(0).unwrap();
        let xs0: LazyVec<_> = xs00.skip(1);
        let ys: LazyVec<_> = shrink(x0.clone()).map(&|x1| {
            let vs = LazyVec::singleton(x1);
            vs.append(xs0);
            vs
        });
        let zs: LazyVec<_> = elems(shrink.clone(), xs0).map(&|xs1: LazyVec<'a, A>| {
            let vs = LazyVec::singleton(x0.clone());
            let vs = vs.append(xs1.clone());
            vs
        });
        let ys = ys.append(zs);
        ys
    }
}

pub fn halves<'a, A>(n: A) -> LazyVec<'a, A>
where
    A: Integer + FromPrimitive + Copy + 'a,
{
    let go = |x0| {
        let zero = num::zero();
        if x0 == zero {
            None
        } else {
            let two = FromPrimitive::from_isize(2).unwrap();
            let x1 = x0 / two;
            Some((x0, x1))
        }
    };
    unfold(go, n)
}

/// Shrink an integral number by edging towards a destination.
pub fn towards<'a, A>(destination: A, x: A) -> LazyVec<'a, A>
where
    A: 'a,
    A: Integer + FromPrimitive + Copy,
{
    if destination == x {
        LazyVec::empty()
    } else {
        // We need to halve our operands before subtracting them as they may be using
        // the full range of the type (i.e. 'MinValue' and 'MaxValue' for 'Int32')
        let two = FromPrimitive::from_isize(2).unwrap();
        let diff = (x / two) - (destination / two);

        cons_nub(destination)(halves(diff).map(&|y| x - y))
    }
}

// TODO: rename to monomorphic variant.
/// Shrink a floating-point number by edging towards a destination.
/// Note we always try the destination first, as that is the optimal shrink.
pub fn towards_float<'a, A: 'a>(destination: A, x: A) -> LazyVec<'a, A>
where
    A: Float + FromPrimitive + Copy + 'a,
{
    if destination == x {
        LazyVec::empty()
    } else {
        let diff = x - destination;
        let go = |n| {
            let x1 = x - n;
            if x1 != x {
                let two = FromPrimitive::from_isize(2).unwrap();
                Some((x1, n / two))
            } else {
                None
            }
        };
        unfold(go, diff)
    }
}

pub fn lazy_vec<'a, A>(xs: LazyVec<'a, A>) -> LazyVec<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
{
    halves(xs.len()).flat_map(&|k| {
        let xs = xs.clone();
        removes(k, xs)
    })
}

pub fn sequence<'a, A, F>(merge: Rc<F>, xs: LazyVec<'a, Tree<'a, A>>) -> Tree<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
    // FIX: This is a bit silly because we don't have a LazyList type.
    F: Fn(LazyVec<'a, Tree<'a, A>>) -> LazyVec<'a, LazyVec<'a, Tree<'a, A>>> + 'a,
{
    let y = xs.clone().map(&|t| tree::outcome(t));
    let ys = merge(xs).map(&|v| sequence(merge.clone(), v));
    Tree::new(y, ys)
}

pub fn sequence_list<'a, A>(xs0: LazyVec<'a, Tree<'a, A>>) -> Tree<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
{
    sequence(
        Rc::new(move |xs: LazyVec<'a, Tree<'a, A>>| {
            let ys = xs.clone();
            let shrinks = lazy_vec(xs);
            let elems = elems(Rc::new(move |t| tree::shrinks(t)), ys);
            let shrinks = shrinks.append(elems);
            shrinks
        }),
        xs0,
    )
}

pub fn sequence_elems<'a, A>(xs0: LazyVec<'a, Tree<'a, A>>) -> Tree<'a, LazyVec<'a, A>>
where
    A: Clone + 'a,
{
    sequence(
        Rc::new(move |xs| elems(Rc::new(move |t| tree::shrinks(t)), xs)),
        xs0,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn towards_works() {
        let f = |x| towards(3, x);
        assert_eq!(f(100), lazy_vec![3, 51, 76, 88, 94, 97, 99]);
    }

    #[test]
    fn towards_float_works() {
        let f = |x| towards_float(100.0, x);

        let expected = lazy_vec![
            100.0,
            300.0,
            400.0,
            450.0,
            475.0,
            487.5,
            493.75,
            496.875,
            498.4375,
            499.21875,
            499.609375,
            499.8046875,
            499.90234375,
            499.951171875,
            499.9755859375,
            499.98779296875,
            499.993896484375,
            499.9969482421875,
            499.99847412109375,
            499.9992370605469,
            499.99961853027344,
            499.9998092651367,
            499.99990463256836,
            499.9999523162842,
            499.9999761581421,
            499.99998807907104,
            499.9999940395355,
            499.99999701976776,
            499.9999985098839,
            499.99999925494194,
            499.99999962747097,
            499.9999998137355,
            499.99999990686774,
            499.99999995343387,
            499.99999997671694,
            499.99999998835847,
            499.99999999417923,
            499.9999999970896,
            499.9999999985448,
            499.9999999992724,
            499.9999999996362,
            499.9999999998181,
            499.99999999990905,
            499.9999999999545,
            499.99999999997726,
            499.99999999998863,
            499.9999999999943,
            499.99999999999716,
            499.9999999999986,
            499.9999999999993,
            499.99999999999966,
            499.99999999999983,
            499.9999999999999,
            499.99999999999994,
        ];

        assert_eq!(f(500.0), expected);
    }
}
