use gen::Gen;
use std::rc::Rc;
use std::fmt::Display;

#[derive(Clone)]
pub struct Journal(Vec<String>);

// TODO: Rename. Maybe TestResult?
#[derive(Clone)]
pub enum Result<A>
where A: Clone
{
    Failure,
    Discard,
    Success(A),
}

#[derive(Clone)]
pub struct Property<'a, A>(Gen<'a, (Journal, Result<A>)>)
where A: Clone;

pub enum Status {
    Failed((isize, Journal)), // isize -> Shrinks
    GaveUp,
    Ok,
}

pub struct Report {
    pub tests: isize, // isize -> tests
    pub discards: isize, // isize -> discards
    pub status: Status,
}

mod tuple {
    pub fn first<'a, F, A, B, C>(f: F, x: A, y: B) -> (C, B)
        where A: Clone,
              B: Clone,
              C: Clone,
              F: Fn(A) -> C,
    {
        (f(x), y)
    }

    pub fn second<'a, F, A, B, C>(f: F, x: A, y: B) -> (A, C)
        where A: Clone,
              B: Clone,
              C: Clone,
              F: Fn(B) -> C,
    {
        (x, f(y))
    }
}

mod journal {
    use super::*;

    pub fn from_list(xs: Vec<String>) -> Journal {
        Journal(xs)
    }

    pub fn to_vec(Journal(xs): Journal) -> Vec<String> {
        xs
    }

    pub fn empty() -> Journal {
        Journal(vec![])
    }

    pub fn singleton(x: String) -> Journal {
        Journal(vec![x])
    }

    pub fn delayed_singleton<F>(x: &F) -> Journal
        where F: Fn() -> String,
    {
        eprintln!("TODO: delayed_singleton");
        unimplemented!()
    }

    pub fn append(Journal(xs): Journal, Journal(ys): Journal) -> Journal {
        let zs = xs.into_iter().chain(ys).collect();
        Journal(zs)
    }
}

mod result {
    use super::*;

    pub fn map<F, A, B>(f: F, r: Result<A>) -> Result<B>
        where F: Fn(A) -> B,
              A: Clone,
              B: Clone,
    {
        match r {
            Result::Failure =>
                Result::Failure,
            Result::Discard =>
                Result::Discard,
            Result::Success(x) =>
                Result::Success(f(x)),
        }
    }

    pub fn filter<F, A>(f: F, r: Result<A>) -> Result<A>
        where F: Fn(&A) -> bool,
              A: Clone,
    {
        match r {
            Result::Failure =>
                Result::Failure,
            Result::Discard =>
                Result::Discard,
            Result::Success(x) =>
                if f(&x) {
                    Result::Success(x)
                } else {
                    Result::Discard
                },
        }
    }

    pub fn is_failure<A>(r: Result<A>) -> bool
        where A: Clone,
    {
        match r {
            Result::Failure =>
                true,
            Result::Discard =>
                false,
            Result::Success(x) =>
                false,
        }
    }
}

mod pretty {
    use super::*;

    // isize -> Tests
    pub fn render_tests(x: isize) -> String {
        match x {
            1 => "1 test".to_string(),
            n => format!("{} tests", n),
        }
    }

    // isize -> Discards
    pub fn render_discards(x: isize) -> String {
        match x {
            1 => "1 discard".to_string(),
            n => format!("{} discards", n),
        }
    }

    // isize -> Discards
    pub fn render_and_discards(x: isize) -> String {
        match x {
            0 => "".to_string(),
            1 => " and 1 discard".to_string(),
            n => format!(" and {} discards", n),
        }
    }

    // isize -> Shrinks
    pub fn render_and_shrinks(x: isize) -> String {
        match x {
            0 => "".to_string(),
            1 => " and 1 shrink".to_string(),
            n => format!(" and {} shrinks", n),
        }
    }

    // isize -> Tests
    pub fn render_ok(tests: isize) -> String {
        format!("+++ OK, passed {}.", render_tests(tests))
    }

    // isize -> Tests, isize -> Discards
    pub fn render_gave_up(tests: isize, discards: isize) -> String {
        format!("*** Gave up after {}, passed {}.", render_discards(discards), render_tests(tests))
    }

    // isize -> Tests, isize -> Discards, isize -> Shrinks
    pub fn render_failed(tests: isize, discards: isize, shrinks: isize, journal: Journal) -> String {
        let mut s = format!("*** Failed! Falsifiable (after {}{}{}):",
            render_tests(tests),
            render_and_shrinks(shrinks),
            render_discards(discards),
        );
        journal::to_vec(journal).iter().for_each(|entry| s.push_str(entry));
        // discard extra newline?
        s.truncate(s.len()-1);
        s
    }
}

mod report {
    use super::*;

    pub fn render(report: Report) -> String {
        match report.status {
            Status::Ok =>
                pretty::render_ok(report.tests),
            Status::GaveUp =>
                pretty::render_gave_up(report.tests, report.discards),
            Status::Failed((shrinks, journal)) =>
                pretty::render_failed(report.tests, report.discards, shrinks, journal),
        }
    }

    // We could do this if we implemented the exceptions as Error.
    // then we could return them via Result.
    // Which would be a nice idiomatic rust pattern.
    //pub fn try_raise(report: Report) -> String {
        //match report.status {
            //Status::Ok =>
                //pretty::render_ok(report.tests),
            //Status::GaveUp =>
                //pretty::render_gave_up(report.tests, report.discards),
            //Status::Failed((shrinks, journal)) =>
                //pretty::render_failed(report.tests, report.discards, shrinks, journal),
        //}
    //}
}

mod property {
    use super::*;
    use crate::gen;

    pub fn from_gen<A>(x: Gen<(Journal, Result<A>)>) -> Property<A>
        where A: Clone,
    {
        Property(x)
    }

    pub fn to_gen<A>(Property(x): Property<A>) -> Gen<(Journal, Result<A>)>
        where A: Clone,
    {
        x
    }

    // TODO:
    // try_finally
    // try_with

    pub fn delay<'a, F, A>(f: F) -> Property<'a, A>
        where A: Clone + 'a,
              F: Fn() -> Property<'a, A> + 'a,
    {
        from_gen(gen::delay(Box::new(move || to_gen(f()))))
    }

    // TODO
    // using

    pub fn filter<'a, F, A>(p: F, m: Property<'a, A>) -> Property<'a, A>
        where A: Clone,
              F: Fn(A) -> bool,
    {
        // TODO: Tuple mod.
        //from_gen(gen::map(Rc::new(move |x| second(result::filter(p(x)))), to_gen(m)))
        unimplemented!()
    }

    pub fn from_result<'a, A>(x: Result<A>) -> Property<'a, A>
        where A: Clone + 'a,
    {
        from_gen(
            gen::constant(
                (journal::empty(), x)
            )
        )
    }

    pub fn failure<'a>() -> Property<'a, ()> {
        from_result(Result::Failure)
    }

    pub fn discard<'a>() -> Property<'a, ()> {
        from_result(Result::Discard)
    }

    pub fn success<'a, A>(x: A) -> Property<'a, A>
        where A: Clone + 'a,
    {
        from_result(Result::Success(x))
    }

    pub fn from_bool<'a>(x: bool) -> Property<'a, ()> {
        if x {
            success(())
        } else {
            failure()
        }
    }

    pub fn counter_example<'a, F>(msg: &F) -> Property<'a, ()>
        where F: Fn() -> String,
    {
        let inner = (journal::delayed_singleton(msg), Result::Success(()));
        from_gen(gen::constant(inner))
    }

    fn map_gen<'a, A, B, F>(f: F, x: Property<'a, A>) -> Property<'a, B>
        where F: Fn(Gen<'a, (Journal, Result<A>)>) -> Gen<(Journal, Result<B>)>,
              A: Clone,
              B: Clone,
    {
        from_gen(f(to_gen(x)))
    }

    pub fn map<'a, F, A, B>(f: F, x: Property<'a, A>) -> Property<'a, B>
        where F: Fn(A) -> B,
              A: Clone,
              B: Clone,
    {
        //let composed = |f, x| {
            //map_gen(f, gen::map(f, second(result::map(f, y))))
        //};
        //composed(f, x)
        // TODO: Needs tuple module.
        unimplemented!()
    }

    fn bind_gen<'a, F, A, B>(m: Gen<'a, (Journal, Result<A>)>, k: F) -> Gen<'a, (Journal, Result<B>)>
        where A: Clone + 'a,
              B: Clone + 'a,
              F: Fn(A) -> Gen<'a, (Journal, Result<B>)> + 'a,
    {
        gen::bind(m, Rc::new(move |(journal, result): (Journal, Result<A>)| {
            match result {
                Result::Failure =>
                    gen::constant((journal, Result::Failure)),
                Result::Discard =>
                    gen::constant((journal, Result::Discard)),
                Result::Success(x) => {
                    gen::map(Rc::new(
                            move |(j, r)| {
                                let journal = journal.clone();
                                tuple::first(
                                    move |j1| {
                                        journal::append(journal.clone(), j1)
                                    }, j, r)
                            }), k(x))
                },
            }
        }))
    }

    pub fn bind<'a, F, A, B>(m: Property<'a, A>, k: F) -> Property<'a, B>
        where
            F: Fn(A) -> Property<'a, B> + 'a,
            A: Clone + 'a,
            B: Clone + 'a,
    {
        from_gen(bind_gen(to_gen(m), move |x| to_gen(k(x))))
    }

    pub fn for_all<'a, F, A, B>(gen: Gen<'a, A>, k: &'a F) -> Property<'a, B>
        where
            F: Fn(A) -> Property<'a, B> + 'a,
            A: Clone + Display + 'a,
            B: Clone + 'a,
    {
        let prepend = Rc::new(move |x: A| {
            // pretend things don't panic.
            to_gen(bind(counter_example(&|| format!("{}", x)), move |_| k(x.clone())))
        });
        from_gen(gen::bind(gen, prepend))
    }

    pub fn for_all_tick<'a, A>(gen: Gen<'a, A>) -> Property<'a, A>
        where
            A: Clone + Display + 'a,
    {
        for_all(gen, &|x: A| success(x))
    }
}
