#[allow(dead_code)]
pub fn towards<'a, A: 'a>(destination: A) -> Box<Fn(A) -> Vec<A> + 'a>
where
    A: PartialEq,
{
    let towards_do = move |x: A| {
        if destination == x {
            Vec::new()
        } else {
            Vec::new()
        }
    };
    Box::new(towards_do)
}
