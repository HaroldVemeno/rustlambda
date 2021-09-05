use empty_box::EmptyBox;
use std::error;
use std::fmt;

use super::util::*;
use super::EvalError;
use crate::expr::{Def, Defs, Expr};

#[derive(Debug, Default, Clone, Copy)]
struct Stats {
    reduced: bool,
    betas: u32,
    etas: u32,
    max_depth: u32,
    depth: u32,
    size: u32,
    max_size: u32,
}

pub fn reduce(
    mut expr: Box<Expr>,
    defs: &Defs,
    print_info: bool,
) -> Result<Box<Expr>, Box<dyn error::Error>> {
    let max_iterations = 10000000;
    let max_size = 10000000;

    let mut stats = Stats::default();
    for i in 1..=max_iterations {
        stats.reduced = false;
        stats.size = 0;
        expr = do_reduce(expr, defs, &mut stats);
        if stats.size > stats.max_size {
            stats.max_size = stats.size
        }
        //eprintln!("Reduce: {}", expr);
        if !stats.reduced {
            break;
        }
        //let expr_size = expr.size();
        //debug_assert_eq!(expr_size, stats.size);
        if stats.size > max_size {
            return Err(EvalError::boxed(format!(
                "Size outgrew maximum size: {} out of {}",
                stats.size, max_size
            )));
        } else if i == max_iterations {
            return Err(EvalError::boxed(format!(
                "Iteration limit reached: {}",
                max_iterations
            )));
        }
    }
    if print_info {
        eprintln! {"{}", stats}
    }
    Ok(expr)
}

fn do_reduce(expr: Box<Expr>, defs: &Defs, st: &mut Stats) -> Box<Expr> {
    st.depth += 1;
    st.size += 1;
    if st.depth > st.max_depth {
        st.max_depth = st.depth
    }
    //println!("\tdo_reduce: {}", expr);

    use Expr::*;
    let (ex, eb) = EmptyBox::take(expr);
    // TODO: recursive -> iterative for trivial cases
    let result = match ex {
        // Irreducable
        Variable(_) => ex,
        Name(ref s) => {
            if let Some(Def { value }) = defs.get(s) {
                st.reduced = true;
                st.size -= 1;
                *value.clone()
            } else if let Ok(n) = s.parse() {
                st.reduced = true;
                st.size -= 1;
                *Expr::church_num(n)
            } else {
                ex
            }
        }

        // Eta reduction:
        //   Reduce(\a.Ea)
        //   a is not free in E
        //=> Reduce(E)
        Abstr(var, box Appl(rest, box Variable(last)))
            if var == last && !rest.unbounds().contains(&var) =>
        {
            st.etas += 1;
            st.reduced = true;
            *do_reduce(rest, defs, st)
        }
        //   Reduce[\a.E]  =>  \a.Reduce[E]
        Abstr(var, body) => {
            let e = do_reduce(body, defs, st);
            Abstr(var, e)
        }

        // Beta reduction:
        //   Reduce[(\x.A)B]  => Reduce[A[x->B]]
        Appl(box Abstr(from, body), to) => {
            st.betas += 1;
            st.reduced = true;
            let res = beta_reduce(body, from, to);
            st.size += res.size() - 1;
            *res
        }
        //   Reduce[AB]
        Appl(a, to) => {
            let sz = st.size;
            let red_box = do_reduce(a, defs, st);
            let (reduced_a, red_eb) = EmptyBox::take(red_box);
            match reduced_a {
                //   if Reduce[A] => \x.C
                //        Beta reduction:
                //        Reduce[AB] => Reduce[(\x.C)B] => Reduce[C[x->B]]
                Abstr(from, body) => {
                    st.betas += 1;
                    st.reduced = true;
                    let res = beta_reduce(body, from, to);
                    st.size = sz + res.size() - 1;
                    *res
                }
                //   else Reduce[AB] => (Reduce[A])(Reduce[B])
                other => {
                    let e = do_reduce(to, defs, st);
                    st.size -= 1;
                    Appl(red_eb.put(other), e)
                }
            }
        }
    };
    st.depth -= 1;
    eb.put(result)
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Stats {
            betas,
            etas,
            max_depth,
            max_size,
            ..
        } = *self;
        writeln!(
            f,
            r#"Stats:
	Beta reductions: {}
	Eta reductions: {}
	Maximum depth: {}
	Maximum size: {}"#,
            betas, etas, max_depth, max_size
        )
    }
}
