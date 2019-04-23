#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Seed {
    value: u64,
    gamma: u64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub() {
        assert_eq!(1 + 1, 2);
    }
}
