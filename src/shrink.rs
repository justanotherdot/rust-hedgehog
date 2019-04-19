extern crate num;

use self::num::{FromPrimitive, Num};

// This probably could be optimised for an eager language. by simply manipulating the vector
// directly and doing the inner check, rather than returning the function here for use in a
// pipeline a la the F# port.
fn cons_nub<'a, A: 'a>(x: A) -> Box<Fn(Vec<A>) -> Vec<A> + 'a>
where
    A: Num + FromPrimitive + Copy,
{
    let cons_nub_do = move |ys0: Vec<A>| {
        let mut ys1 = ys0.clone();
        if ys1.is_empty() {
            vec![]
        } else {
            let y = ys1.remove(0);
            if x == y {
                ys0
            } else {
                ys1.insert(0, y);
                ys1.insert(0, x);
                ys1
            }
        }
    };
    Box::new(cons_nub_do)
}

// TODO: This function needs testing and verification.
fn unfoldr<A, B>(f: Box<Fn(B) -> Option<(A, B)>>, b0: B) -> Vec<A> {
    match f(b0) {
        Some((a, b1)) => {
            let mut v = unfoldr(f, b1);
            v.insert(0, a); // XXX Always shifts values over on each fn call.
            v
        }
        None => vec![],
    }
}

#[allow(dead_code)]
pub fn halves<A>(n: A) -> Vec<A>
where
    A: Num + FromPrimitive + Copy,
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
    unfoldr(Box::new(go), n)
}

#[allow(dead_code)]
/// Shrink an integral number by edging towards a destination.
pub fn towards<'a, A: 'a>(destination: A) -> Box<Fn(A) -> Vec<A> + 'a>
where
    A: Num + FromPrimitive + Copy,
{
    let towards_do = move |x: A| {
        if destination == x {
            Vec::new()
        } else {
            let two = FromPrimitive::from_isize(2).unwrap();
            let diff = (x / two) - (destination / two);

            cons_nub(destination)(halves(diff).into_iter().map(|y| x - y).collect())
        }
    };
    Box::new(towards_do)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn towards_shrinks() {
        let f = towards(3);
        assert_eq!(f(100), vec![3, 51, 76, 88, 94, 97, 99]);
    }
}
