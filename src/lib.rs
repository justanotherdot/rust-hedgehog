pub mod lazy {
    pub struct Thunk<'a, T> {
        pub value: Option<T>,
        closure: Box<'a + Fn() -> T>,
    }

    impl<'a, T: Clone> Thunk<'a, T> {
        pub fn new<F>(closure: F) -> Thunk<'a, T>
        where
            F: 'a + Fn() -> T,
        {
            Thunk {
                closure: Box::new(closure),
                value: None,
            }
        }

        pub fn force(&mut self) -> &mut Self {
            if self.value.is_none() {
                self.value = Some((self.closure)());
            }
            self
        }
    }

    #[allow(dead_code)]
    pub struct Tree<'a, T> {
        pub thunk: Thunk<'a, T>,
        children: Vec<Tree<'a, T>>,
    }

    impl<'a, T: 'a + Clone> Tree<'a, T> {
        pub fn singleton(value: T) -> Tree<'a, T> {
            Tree {
                thunk: Thunk::new(move || value.clone()),
                children: vec![],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lazy::{Thunk, Tree};
    use std::time::SystemTime;

    #[test]
    fn thunks_defer_application_until_forced() {
        let mut t = Thunk::new(|| SystemTime::now());
        let v = t.force().value;
        assert!(v != Some(SystemTime::now()));
    }

    #[test]
    fn thunks_memoize_values() {
        let mut t = Thunk::new(|| SystemTime::now());
        let p = t.force().value;
        let q = t.force().value;
        assert_eq!(p, q);
    }

    #[test]
    fn rose_trees_hold_lazy_values() {
        let mut tree = Tree::singleton(42);
        let p = tree.thunk.force().value;
        let q = tree.thunk.force().value;
        assert_eq!(p, q);
    }
}
