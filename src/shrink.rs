extern crate num;

use self::num::{Float, FromPrimitive, Integer};
use crate::tree;
use crate::tree::Tree;
use std::rc::Rc;

// This probably could be optimised for an eager language. by simply manipulating the vector
// directly and doing the inner check, rather than returning the function here for use in a
// pipeline a la the F# port.
fn cons_nub<'a, A: 'a>(x: A) -> Box<dyn Fn(Vec<A>) -> Vec<A> + 'a>
where
    A: Integer + FromPrimitive + Copy,
{
    let cons_nub_do = move |ys0: Vec<A>| match ys0.first() {
        None => vec![],
        Some(&y) if x == y => ys0,
        Some(_) => {
            let mut ys1 = ys0.clone();
            ys1.insert(0, x);
            ys1
        }
    };
    Box::new(cons_nub_do)
}

fn unfold<A, B, F>(f: F, b0: B) -> Vec<A>
where
    F: Fn(B) -> Option<(A, B)>,
{
    let mut acc = vec![];
    let mut b = b0;
    loop {
        if let Some((a, b1)) = f(b) {
            acc.push(a);
            b = b1;
            continue;
        } else {
            break;
        }
    }
    acc
}

pub fn removes<A, B>(k0: B, xs0: Vec<A>) -> Vec<Vec<A>>
where
    B: Integer + FromPrimitive + Copy,
{
    fn loop0<C, D>(k: C, n: C, xs: Vec<D>) -> Vec<Vec<D>>
    where
        C: Integer + FromPrimitive + Copy,
    {
        let hd = &xs.into_iter().take(1).collect::<Vec<_>>()[0];
        let tl: Vec<_> = xs.into_iter().skip(1).collect();
        if k > n {
            vec![]
        } else if tl.is_empty() {
            vec![vec![]]
        } else {
            let mut inner: Vec<_> = loop0(k, n - k, tl)
                .into_iter()
                .map(|mut x| {
                    x.push(hd);
                    x
                })
                .collect();
            inner.insert(0, tl);
            inner
        }
    }
    let gen_len = FromPrimitive::from_usize(xs0.len()).unwrap();
    loop0(k0, gen_len, xs0)
}

pub fn elems<A, F>(shrink: Rc<F>, xs00: Vec<A>) -> Vec<Vec<A>>
where
    F: Fn(A) -> Vec<A>,
{
    if xs00.is_empty() {
        vec![]
    } else {
        let xs01 = xs00.into_iter().take(1).collect::<Vec<_>>();
        let x0 = xs01.get(0).unwrap();
        let xs0: Vec<_> = xs00.into_iter().skip(1).collect();
        let mut ys: Vec<_> = shrink(&x0)
            .into_iter()
            .map(|x1| {
                let mut vs = vec![x1];
                vs.append(&mut xs0);
                vs
            })
            .collect();
        let mut zs: Vec<_> = elems(shrink.clone(), xs0)
            .into_iter()
            .map(|xs1| {
                let mut vs = vec![x0];
                vs.append(&mut xs1);
                vs
            })
            .collect();
        ys.append(&mut zs);
        ys
    }
}

pub fn halves<A>(n: A) -> Vec<A>
where
    A: Integer + FromPrimitive + Copy,
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
pub fn towards<'a, A>(destination: A, x: A) -> Vec<A>
where
    A: 'a,
    A: Integer + FromPrimitive + Copy,
{
    if destination == x {
        vec![]
    } else {
        // We need to halve our operands before subtracting them as they may be using
        // the full range of the type (i.e. 'MinValue' and 'MaxValue' for 'Int32')
        let two = FromPrimitive::from_isize(2).unwrap();
        let diff = (x / two) - (destination / two);

        cons_nub(destination)(halves(diff).into_iter().map(|y| x - y).collect())
    }
}

/// Shrink a floating-point number by edging towards a destination.
/// Note we always try the destination first, as that is the optimal shrink.
pub fn towards_float<'a, A: 'a>(destination: A, x: A) -> Vec<A>
where
    A: Float + FromPrimitive + Copy,
{
    if destination == x {
        Vec::new()
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

// n.b. previously `list'
pub fn vec<A>(xs: Vec<A>) -> Vec<Vec<A>> {
    halves(xs.len())
        .into_iter()
        .flat_map(move |k| removes(k, xs))
        .collect()
}

pub fn sequence<A, F>(
    merge: Rc<F>,
    xs: Box<dyn Iterator<Item = Tree<A>>>,
) -> Tree<Box<dyn Iterator<Item = A>>>
where
    F: Fn(
        Box<dyn Iterator<Item = Tree<A>>>,
    ) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = Tree<A>>>>>,
{
    let y = xs.map(|t| tree::outcome(&t));
    let ys = Box::new(merge(xs).into_iter().map(|v| sequence(merge.clone(), v)));
    Tree::new(y, ys)
}

pub fn sequence_list<A>(xs0: Vec<Tree<A>>) -> Tree<Vec<A>> {
    sequence(
        Rc::new(move |xs: Vec<Tree<A>>| {
            let ys = xs;
            let mut shrinks = vec(xs);
            let mut elems = elems(Rc::new(move |t| tree::shrinks(t)), ys);
            shrinks.append(&mut elems);
            shrinks
        }),
        xs0,
    )
}

pub fn sequence_elems<A>(xs0: Vec<Tree<A>>) -> Tree<Vec<A>> {
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
        assert_eq!(f(100), vec![3, 51, 76, 88, 94, 97, 99]);
    }

    #[test]
    fn towards_float_works() {
        let f = |x| towards_float(100.0, x);
        let expected = vec![
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
