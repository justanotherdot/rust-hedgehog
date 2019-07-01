extern crate num;

use self::num::{Float, FromPrimitive, Integer};

// This probably could be optimised for an eager language. by simply manipulating the vector
// directly and doing the inner check, rather than returning the function here for use in a
// pipeline a la the F# port.
fn cons_nub<'a, A: 'a>(x: A) -> Box<Fn(Vec<A>) -> Vec<A> + 'a>
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

// TODO: This function needs testing and verification.
// TODO: This function could just be a loop.
fn unfold<A, B>(f: impl Fn(B) -> Option<(A, B)>, b0: B) -> Vec<A> {
    match f(b0) {
        Some((a, b1)) => {
            let mut v = unfold(f, b1);
            v.insert(0, a); // XXX Always shifts values over on each fn call.
            v
        }
        None => vec![],
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
pub fn towards<'a, A>(destination: A) -> impl Fn(A) -> Vec<A>
where
    A: 'a,
    A: Integer + FromPrimitive + Copy,
{
    let towards_do = move |x: A| {
        if destination == x {
            vec![]
        } else {
            // We need to halve our operands before subtracting them as they may be using
            // the full range of the type (i.e. 'MinValue' and 'MaxValue' for 'Int32')
            let two = FromPrimitive::from_isize(2).unwrap();
            let diff = (x / two) - (destination / two);

            cons_nub(destination)(halves(diff).into_iter().map(|y| x - y).collect())
        }
    };
    towards_do
}

// TODO: rename to monomorphic variant.
/// Shrink a floating-point number by edging towards a destination.
/// Note we always try the destination first, as that is the optimal shrink.
pub fn towards_float<'a, A: 'a>(destination: A) -> impl Fn(A) -> Vec<A>
where
    A: Float + FromPrimitive + Copy,
{
    let towards_do = move |x: A| {
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
    };
    towards_do
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn towards_works() {
        let f = towards(3);
        assert_eq!(f(100), vec![3, 51, 76, 88, 94, 97, 99]);
    }

    #[test]
    fn towards_float_works() {
        let f = towards_float(100.0);

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
