use num::{Bounded, Float, FromPrimitive, Integer, Num, ToPrimitive};
use std::rc::Rc;

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
pub struct Size(pub isize);

#[derive(Clone)]
pub struct Range<'a, A: 'a>(A, Rc<dyn Fn(Size) -> (A, A) + 'a>);

impl<'a, A> Range<'a, A> {
    pub fn map<F, B>(f: F, Range(z, g): Range<'a, A>) -> Range<'a, B>
    where
        F: Fn(A) -> B + 'a,
    {
        Range(
            f(z),
            Rc::new(move |sz| {
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
    Range(x.clone(), Rc::new(move |_| (x.clone(), x.clone())))
}

pub fn constant<'a, A>(x: A, y: A) -> Range<'a, A>
where
    A: Clone,
{
    constant_from(x.clone(), x, y)
}

pub fn constant_from<'a, A>(z: A, x: A, y: A) -> Range<'a, A>
where
    A: Clone,
{
    Range(z, Rc::new(move |_| (x.clone(), y.clone())))
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
        Rc::new(move |sz| {
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

pub fn linear_frac<'a, A>(x: A, y: A) -> Range<'a, A>
where
    A: Num + Clone + Ord + FromPrimitive,
{
    linear_frac_from(x.clone(), x, y)
}

pub fn linear_frac_from<'a, A>(z: A, x: A, y: A) -> Range<'a, A>
where
    A: Num + Ord + FromPrimitive + Clone,
{
    Range(
        z.clone(),
        Rc::new(move |sz| {
            let x_sized = clamp(
                x.clone(),
                y.clone(),
                scale_linear_frac(sz.clone(), z.clone(), x.clone()),
            );
            let y_sized = clamp(
                x.clone(),
                y.clone(),
                scale_linear_frac(sz.clone(), z.clone(), y.clone()),
            );
            (x_sized, y_sized)
        }),
    )
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

// FIXME All these frac ones need to be Ratio
// although Ratio and Rational are not traits!
pub fn scale_linear_frac<'a, A>(sz0: Size, z0: A, n0: A) -> A
where
    A: Num + Ord + FromPrimitive + Clone,
{
    let zero = FromPrimitive::from_isize(0).unwrap();
    let ninety_nine_sz = FromPrimitive::from_isize(99).unwrap();
    let sz = std::cmp::max(zero, std::cmp::min(ninety_nine_sz, sz0));
    let sz1: A = FromPrimitive::from_isize(sz.0).unwrap();
    let ninety_nine: A = FromPrimitive::from_isize(99).unwrap();
    let diff = (n0 - z0.clone()) * (sz1 / ninety_nine);
    z0 + diff
}

// FIXME I'm not even sure how this works with the constraint of `Integer` + `Float`
// I'm guessing here that if we can strip our dependency on `num` for generalised numbers
// and simply work out how to do it safely within the context of type assertions
// that rust will figure it out for us.
pub fn exponential<'a, A>(x: A, y: A) -> Range<'a, A>
where
    A: Integer + Float + Clone + ToPrimitive + FromPrimitive,
{
    exponential_from(x.clone(), x, y)
}

pub fn exponential_from<'a, A>(z: A, x: A, y: A) -> Range<'a, A>
where
    A: Clone + Ord + Integer + Float + ToPrimitive + FromPrimitive,
{
    Range(
        z.clone(),
        Rc::new(move |sz| {
            let x_sized = clamp(
                x.clone(),
                y.clone(),
                scale_exponential(sz, z.clone(), x.clone()),
            );
            let y_sized = clamp(
                x.clone(),
                y.clone(),
                scale_exponential(sz, z.clone(), y.clone()),
            );
            (x_sized, y_sized)
        }),
    )
}

pub fn exponential_bounded<'a, A>() -> Range<'a, A>
where
    A: Bounded + Integer + FromPrimitive + Float,
{
    let zero = FromPrimitive::from_isize(0).unwrap();
    exponential_from(zero, Bounded::min_value(), Bounded::max_value())
}

pub fn exponential_float<'a, A>(x: A, y: A) -> Range<'a, A>
where
    A: Float + Ord + FromPrimitive,
{
    exponential_float_from(x.clone(), x, y)
}

pub fn exponential_float_from<'a, A>(z: A, x: A, y: A) -> Range<'a, A>
where
    A: Float + Ord + FromPrimitive,
{
    Range(
        z.clone(),
        Rc::new(move |sz| {
            let x_sized = clamp(
                x.clone(),
                y.clone(),
                scale_exponential_float(sz, z.clone(), x.clone()),
            );
            let y_sized = clamp(
                x.clone(),
                y.clone(),
                scale_exponential_float(sz, z.clone(), y.clone()),
            );
            (x_sized, y_sized)
        }),
    )
}

pub fn scale_exponential<'a, A>(sz: Size, z0: A, n0: A) -> A
where
    A: Integer + Float + FromPrimitive + ToPrimitive,
{
    let z: A = FromPrimitive::from_f64(z0.to_f64().unwrap()).unwrap();
    let n: A = FromPrimitive::from_f64(n0.to_f64().unwrap()).unwrap();

    FromPrimitive::from_f64(scale_exponential_float(sz, z, n).to_f64().unwrap().round()).unwrap()
}

pub fn scale_exponential_float<'a, A>(sz0: Size, z: A, n: A) -> A
where
    A: Float + FromPrimitive,
{
    let zero = FromPrimitive::from_isize(0).unwrap();
    let ninety_nine = FromPrimitive::from_isize(99).unwrap();
    let one = FromPrimitive::from_isize(1).unwrap();
    let sz = clamp(zero, ninety_nine, sz0);
    let x: A = FromPrimitive::from_isize((sz / ninety_nine).0).unwrap();
    let diff = (((n - z).abs() + one).powf(x) - one) * (n - z).signum();
    z + diff
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn stub() {
        assert_eq!(1 + 1, 2);
    }
}
