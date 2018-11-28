pub mod lazy {
    use std::rc::Rc;
    use std::cell::RefCell;

    pub struct Thunk<'a, T> {
        thunk: RefCell<Option<T>>,
        think: Box<'a + Fn() -> T>,
    }

    impl <'a, T: Clone> Thunk<'a, T> {
        pub fn new<F>(clsr: F) -> Thunk<'a, T> where F: 'a + Fn() -> T {
            Thunk {
                think: Box::new(clsr),
                thunk: RefCell::new(None),
            }
        }

        pub fn force(&self) -> T {
            if self.thunk.borrow().is_none() {
                let rv = (self.think)();
                self.thunk.replace(Some(rv));
            }
            self.thunk
                .clone()
                .into_inner()
                .unwrap()
        }
    }

    #[derive(Clone)]
    pub struct ConsCell<'a, T> {
        v: Option<T>,
        tail: Option<Stream<'a, T>>,
    }

    impl <'a, T: Clone> ConsCell<'a, T> {
        pub fn empty() -> ConsCell<'a, T> {
            ConsCell {
                v: None,
                tail: None,
            }
        }

        pub fn new(v: T, tail: Stream<'a, T>) -> ConsCell<'a, T> {
            ConsCell {
                v: Some(v),
                tail: Some(tail),
            }
        }

        pub fn singleton(v: T) -> ConsCell<'a, T> {
            ConsCell {
                v: Some(v),
                tail: None,
            }
        }

        pub fn val(&self) -> Option<T> {
            self.v.clone()
        }

        pub fn tail(self) -> Option<Stream<'a, T>> {
            self.tail
        }
    }

    #[derive(Clone)]
    pub struct Stream<'a, T> {
        // XXX Wow, this is a lot of angle brackets.
        cell: RefCell<Option<Rc<Thunk<'a, ConsCell<'a, T>>>>>
    }

    impl <'a, T: Clone> Stream<'a, T> {
        pub fn empty() -> Stream<'a, T> {
            Stream {
                cell: RefCell::new(None),
            }
        }

        pub fn new<F>(f: F) -> Stream<'a, T> where F: 'a + Fn() -> ConsCell<'a, T> {
            Stream {
                cell: RefCell::new(Some(Rc::new(Thunk::new(f)))),
            }
        }

        pub fn from(strm: Stream<'a, T>) -> Stream<'a, T> {
            let rc = RefCell::new(strm.cell.into_inner());
            Stream {
                cell: rc,
            }
        }

        pub fn is_empty(&self) -> bool {
            self.cell.borrow().is_none()
        }

        fn unwrap_cell(&self) -> Option<ConsCell<'a, T>> {
            match self.clone().cell.into_inner() {
                Some(rc) => unsafe {
                    let rc_ptr = Rc::into_raw(rc);
                    let thunk = &*rc_ptr;
                    Rc::from_raw(rc_ptr);
                    Some(thunk.force())
                },
                None => {
                    None
                },
            }
        }

        pub fn get(&self) -> Option<T> {
            match self.unwrap_cell() {
                Some(cell) => cell.val(),
                None => None
            }
        }

        pub fn tail(&self) -> Stream<'a, T> {
            let old_strm =
                match self.unwrap_cell() {
                    Some(cell) => cell.tail(),
                    None => None,
                };
            match old_strm {
                Some(strm) => strm,
                None => Stream::empty(),
            }
        }
    }

    impl <'a, T: Clone> Iterator for Stream<'a, T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            if self.is_empty() {
                return None;
            }
            let curr = self.get();
            self.cell.replace(self.tail().cell.into_inner());
            curr
        }
    }

    pub struct Tree<'a, T> {
        v: T,
        children: Stream<'a, Tree<'a, T>>
    }
}

#[cfg(test)]
mod tests {
    use lazy::{Thunk, ConsCell, Stream};
    use std::time::SystemTime;
    //use test::Bencher;

    #[test]
    fn thunks_defer_application_until_forced() {
        let t = Thunk::new(|| 5);
        let v = t.force();
        assert_eq!(v, 5);
    }

    #[test]
    fn thunks_memoize_values() {
        let t = Thunk::new(|| SystemTime::now());
        let p = t.force();
        let q = t.force();
        assert_eq!(p, q);
    }

    #[test]
    fn streams_are_lazy_and_possibly_infinite() {
        fn ints_from<'a>(n: usize) -> Stream<'a, usize> {
            Stream::new(move || ConsCell::new(n, ints_from(n+1)))
        }

        let mut strm = ints_from(5);
        let mut i = 5;
        loop {
            if i > 10 {
                break;
            }

            assert_eq!(strm.get(), Some(i));
            i += 1;
            strm = strm.tail();
        }
    }

    #[test]
    fn streams_are_lazy_and_possibly_finite() {
        fn ints_from_to<'a>(n: usize, m: usize) -> Stream<'a, usize> {
            if n > m {
                return Stream::empty();
            }
            Stream::new(move || ConsCell::new(n, ints_from_to(n+1, m)))
        }

        let mut strm = ints_from_to(5, 7);
        let mut i = 5;
        loop {
            if i > 7 {
                assert_eq!(strm.get(), None);
                break;
            }

            assert_eq!(strm.get(), Some(i));
            i += 1;
            strm = strm.tail();
        }
    }

    #[test]
    fn infinite_streams_impl_iterators() {
        fn ints_from<'a>(n: usize) -> Stream<'a, usize> {
            Stream::new(move || ConsCell::new(n, ints_from(n+1)))
        }

        let strm = ints_from(5);
        let mut i = 5;
        for x in strm {
            if i > 100 {
                break;
            }

            assert_eq!(x, i);
            i += 1;
        }
    }

    #[test]
    fn finite_streams_impl_iterators() {
        fn ints_from_to<'a>(n: usize, m: usize) -> Stream<'a, usize> {
            if n > m {
                return Stream::empty();
            }
            Stream::new(move || ConsCell::new(n, ints_from_to(n+1, m)))
        }

        let strm = ints_from_to(5, 7);
        let mut i = 5;
        for x in strm {
            assert_eq!(x, i);
            i += 1;
        }
    }

    // Benchmarks.
    // Can run via (after uncommenting benchmarks and feature flag at top of file):
    //
    // ```
    //   $ rustup run nightly cargo bench
    // ```

    //#[bench]
    //fn bench_infinite_streams(b: &mut Bencher) {
        //fn ints_from<'a>(n: usize) -> Stream<'a, usize> {
            //Stream::new(move || ConsCell::new(n, ints_from(n+1)))
        //}

        //let strm = ints_from(5);
        //b.iter(move || {
            //let mut i = 5;
            //for _ in strm.clone() {
                //if i > 100 {
                    //break;
                //}
                //i += 1;
            //}
        //});
    //}

    //#[bench]
    //fn bench_finite_streams(b: &mut Bencher) {
        //fn ints_from_to<'a>(n: usize, m: usize) -> Stream<'a, usize> {
            //if n > m {
                //return Stream::empty();
            //}
            //Stream::new(move || ConsCell::new(n, ints_from_to(n+1, m)))
        //}

        //let strm = ints_from_to(5, 100);
        //b.iter(|| {
            //for _ in strm.clone() {
                //continue;
            //}
        //});
    //}
}
