extern crate num;

use self::num::{FromPrimitive, Num};

#[allow(dead_code)]
pub fn halves<A>(_n: A) -> Vec<A> {
    vec![]
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

            halves(diff).into_iter().map(|y| x - y).collect()
        }
    };
    Box::new(towards_do)
}
