extern crate num;

use self::num::{Bounded, FromPrimitive, Num};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Size {
    un_size: isize,
}

pub struct Range<'a, A: 'a>(A, Box<Fn(Size) -> (A, A) + 'a>);

impl<'a, A> Range<'a, A> {
    pub fn map<F, B>(f: F, Range(z, g): Range<'a, A>) -> Range<'a, B>
    where
        F: Fn(A) -> B + 'a,
    {
        Range(
            f(z),
            Box::new(move |sz| {
                let (a, b) = g(sz);
                (f(a), f(b))
            }),
        )
    }
}

pub fn origin<A>(Range(z, _): Range<A>) -> A {
    z
}

pub fn bounds<A>(sz: Size, Range(_, f): Range<A>) -> (A, A) {
    f(sz)
}

pub fn lower_bound<A>(sz: Size, range: Range<A>) -> A
where
    A: Ord,
{
    let (x, y) = bounds(sz, range);
    std::cmp::min(x, y)
}

pub fn upper_bound<A>(sz: Size, range: Range<A>) -> A
where
    A: Ord,
{
    let (x, y) = bounds(sz, range);
    std::cmp::max(x, y)
}

// FIXME lots of clones here the Haskell variant is probably simply using references to the same
// one. So it might make sense to refactor this as an Rc around A.
pub fn singleton<'a, A>(x: A) -> Range<'a, A>
where
    A: Clone,
{
    Range(x.clone(), Box::new(move |_| (x.clone(), x.clone())))
}

// FIXME lots of clones here the Haskell variant is probably simply using references to the same
// one. So it might make sense to refactor this as an Rc around A.
pub fn constant<'a, A>(x: A, y: A) -> Range<'a, A>
where
    A: Clone,
{
    constant_from(x.clone(), x, y);
    unimplemented!();
}

// FIXME lots of clones here the Haskell variant is probably simply using references to the same
// one. So it might make sense to refactor this as an Rc around A.
pub fn constant_from<'a, A>(z: A, x: A, y: A) -> Range<'a, A>
where
    A: Clone,
{
    Range(z, Box::new(move |_| (x.clone(), y.clone())))
}

pub fn constant_bounded<'a, A>() -> Range<'a, A>
where
    A: Num + Bounded + Clone + FromPrimitive,
{
    constant_from(
        FromPrimitive::from_isize(0).unwrap(),
        Bounded::min_value(),
        Bounded::max_value(),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub() {
        assert_eq!(1 + 1, 2);
    }
}
