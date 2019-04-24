use num::{Bounded, FromPrimitive, Integer, Num};

//#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Num)]
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    ToPrimitive,
    FromPrimitive,
    NumOps,
    NumCast,
    One,
    Zero,
    Num,
)]
pub struct Size(isize);

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

pub fn linear<'a, A>(x: A, y: A) -> Range<'a, A>
where
    A: Integer + Clone + FromPrimitive,
{
    linear_from(x.clone(), x, y)
}

pub fn linear_from<'a, A>(z: A, x: A, y: A) -> Range<'a, A>
where
    A: Integer + Clone + FromPrimitive,
{
    Range(
        z.clone(),
        Box::new(move |sz| {
            let x_sized = clamp(
                x.clone(),
                y.clone(),
                scale_linear(sz.clone(), z.clone(), x.clone()),
            );
            let y_sized = clamp(
                x.clone(),
                y.clone(),
                scale_linear(sz.clone(), z.clone(), y.clone()),
            );

            (x_sized, y_sized)
        }),
    )
}

pub fn linear_bounded<'a, A>() -> Range<'a, A>
where
    A: Bounded + Integer + Clone + FromPrimitive,
{
    let zero = FromPrimitive::from_isize(0).unwrap();
    linear_from(zero, A::min_value(), A::max_value())
}

pub fn clamp<'a, A>(x: A, y: A, n: A) -> A
where
    A: Ord,
{
    if x > y {
        std::cmp::min(x, std::cmp::max(y, n))
    } else {
        std::cmp::min(y, std::cmp::max(x, n))
    }
}

pub fn scale_linear<'a, A>(sz0: Size, z0: A, n0: A) -> A
where
    A: Integer + FromPrimitive + Clone,
{
    let zero = FromPrimitive::from_isize(0).unwrap();
    let ninety_nine_sz = FromPrimitive::from_isize(99).unwrap();
    let sz = std::cmp::max(zero, std::cmp::min(ninety_nine_sz, sz0));
    let sz1 = FromPrimitive::from_isize(sz.0).unwrap();
    let ninety_nine: A = FromPrimitive::from_isize(99).unwrap();
    let (diff, _) = Integer::div_rem(&((n0 - z0.clone()) * sz1), &ninety_nine);
    z0 + diff
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub() {
        assert_eq!(1 + 1, 2);
    }
}
