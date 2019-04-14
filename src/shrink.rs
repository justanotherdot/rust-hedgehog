#[allow(dead_code)]
pub fn towards<A: 'static + PartialEq>(destination: A) -> Box<Fn(A) -> Vec<A>> {
    let towards_do = move |x: A| {
        if destination == x {
            Vec::new()
        } else {
            Vec::new()
        }
    };
    Box::new(towards_do)
}
