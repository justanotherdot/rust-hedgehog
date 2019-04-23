#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Size {
    un_size: isize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub() {
        assert_eq!(1 + 1, 2);
    }
}
