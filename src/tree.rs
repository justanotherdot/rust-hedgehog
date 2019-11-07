use std::fmt;
use std::fmt::{Debug, Display, Write};
use std::rc::Rc;

pub struct Tree<A> {
    value: A,
    pub children: Vec<Tree<A>>,
}

impl<A> Tree<A> {
    pub fn new(value: A, children: Vec<Tree<A>>) -> Self {
        Tree { value, children }
    }

    pub fn singleton(value: A) -> Tree<A> {
        Tree {
            value,
            children: vec![],
        }
    }

    pub fn value(&self) -> A {
        self.value
    }

    pub fn expand<F>(f: Rc<F>, t: &Tree<A>) -> Tree<A>
    where
        F: Fn(&A) -> Vec<A>,
    {
        let mut children: Vec<Tree<A>> = t
            .children
            .iter()
            .map(move |t| Self::expand(f.clone(), t))
            .collect();
        let mut zs = unfold_forest(Rc::new(|x| x), f, &t.value());
        children.append(&mut zs);
        Tree::new(t.value(), children)
    }
}

pub fn bind<A, B, F>(t: &Tree<A>, k: Rc<F>) -> Tree<B>
where
    F: Fn(A) -> Tree<B>,
{
    let x = t.value();
    let xs0 = t.children;
    let mut t1 = k(x);
    let mut xs: Vec<Tree<B>> = xs0.iter().map(|m| bind(m, k.clone())).collect();
    xs.append(&mut t1.children);
    Tree {
        value: t1.value,
        children: xs,
    }
}

pub fn join<A>(tss: &Tree<Tree<A>>) -> Tree<A> {
    bind(tss, Rc::new(move |x| x))
}

pub fn duplicate<A>(t: Tree<A>) -> Tree<Tree<A>> {
    let xs = t.children.into_iter().map(|x| duplicate(x)).collect();
    Tree::new(t, xs)
}

pub fn fold<A, X, B, F, G>(f: &F, g: &G, t: Tree<A>) -> B
where
    F: Fn(A, X) -> B,
    G: Fn(Vec<B>) -> X,
{
    let x = t.value();
    let xs = t.children;
    f(x, fold_forest(f, g, xs))
}

pub fn fold_forest<A, X, B, F, G>(f: &F, g: &G, xs: Vec<Tree<A>>) -> X
where
    F: Fn(A, X) -> B,
    G: Fn(Vec<B>) -> X,
{
    g(xs.into_iter().map(|x| fold(f, g, x)).collect())
}

impl<A: PartialEq> PartialEq for Tree<A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
            && self
                .children
                .iter()
                .zip(&other.children)
                .all(|(x, y)| x.value() == y.value())
    }
}

/// Build a tree from an unfolding function and a seed value.
pub fn unfold<A, B, F, G>(f: Rc<F>, g: Rc<G>, x: &B) -> Tree<A>
where
    F: Fn(&B) -> A,
    G: Fn(&B) -> Vec<B>,
{
    let y = f(x);
    Tree {
        value: y,
        children: unfold_forest(f.clone(), g.clone(), x),
    }
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<A, B, F, G>(f: Rc<F>, g: Rc<G>, x: &B) -> Vec<Tree<A>>
where
    F: Fn(&B) -> A,
    G: Fn(&B) -> Vec<B>,
{
    g(x).iter()
        .map(move |v| unfold(f.clone(), g.clone(), v))
        .collect()
}

/// Alias for `value`.
pub fn outcome<A>(t: &Tree<A>) -> A {
    t.value()
}

pub fn shrinks<A>(t: Tree<A>) -> Vec<Tree<A>> {
    t.children
}

pub fn filter<A, F>(f: Rc<F>, t: Tree<A>) -> Tree<A>
where
    F: Fn(A) -> bool,
{
    Tree::new(t.value(), filter_forest(f.clone(), t.children))
}

pub fn filter_forest<A, F>(f: Rc<F>, xs: Vec<Tree<A>>) -> Vec<Tree<A>>
where
    F: Fn(A) -> bool,
{
    xs.into_iter()
        .filter(|x| f(outcome(x.clone())))
        .map(|x| filter(f.clone(), x))
        .collect()
}

pub fn map<A, B, F>(f: Rc<F>, t: Tree<A>) -> Tree<B>
where
    F: Fn(A) -> B,
{
    let x = f(t.value());
    let xs = t.children.into_iter().map(|c| map(f.clone(), c)).collect();
    Tree::new(x, xs)
}

// should be: shift hd other = zipWith (++) (hd : repeat other)
fn shift(head: &str, other: &str, lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut first = true;
    for line in lines {
        if first {
            first = false;
            out.push(format!("{}{}", head, line));
        } else {
            out.push(format!("{}{}", other, line));
        }
    }
    out
}

fn render_forest_lines<A: Debug>(limit: i16, forest: &[Tree<A>]) -> Vec<String> {
    if limit <= 0 {
        return vec!["...".to_owned()];
    }

    match forest {
        [] => vec![],
        [x] => {
            let s = render_tree_lines(limit - 1, x);
            shift(" └╼", "   ", s)
        }
        xs0 => {
            let (x, xs) = xs0.split_at(1);
            let s0 = render_tree_lines(limit - 1, x.first().unwrap());
            let ss = render_forest_lines(limit, xs);

            let mut s = shift(" ├╼", " │ ", s0);

            s.extend(ss.into_iter());
            s
        }
    }
}

fn render_tree_lines<A: Debug>(limit: i16, x: &Tree<A>) -> Vec<String> {
    let mut children: Vec<String> = render_forest_lines(limit, &x.children);
    let node = format!(" {:?}", x.value());

    children.insert(0, node);
    children
}

impl<A> Display for Tree<A>
where
    A: Copy + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>
    where
        A: Debug,
    {
        for line in render_tree_lines(100, self) {
            // surely a better way for direct Strings
            f.write_str(&line)?;
            f.write_char('\n')?;
        }

        Ok(())
    }
}
