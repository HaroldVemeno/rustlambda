use empty_box::EmptyBox;
use std::collections::HashSet;
use std::error;
use std::panic;

use super::EvalError;
use crate::expr::{size, unbounds_in, Expr};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Stats {
    reduced: bool,
    betas: u32,
    etas: u32,
    //max_depth: u32,
    //depth: u32,
    //size: u32,
}

pub fn reduce(mut expr: Box<Expr>, print_info: bool) -> Result<Box<Expr>, Box<dyn error::Error>> {
    let max_iterations = 100000;
    let max_size = 1000000;

    let mut stats = Stats::default();
    for i in 1..=max_iterations {
        stats.reduced = false;
        expr = do_reduce(expr, &mut stats);
        if !stats.reduced {
            break;
        }
        //println!("Reduce: {}", expr);
        let expr_size = size(&expr);
        if expr_size > max_size {
            return Err(EvalError::boxed(format!(
                "Size outgrew maximum size: {} out of {}",
                expr_size, max_size
            )));
        } else if i == max_iterations {
            return Err(EvalError::boxed(format!(
                "Iteration limit reached: {}",
                max_iterations
            )));
        }
    }
    if print_info {
        println! {"{:?}", stats}
    }
    Ok(expr)
}

fn do_reduce(expr: Box<Expr>, st: &mut Stats) -> Box<Expr> {
    //println!("\tdo_reduce: {}", expr);

    use Expr::*;
    let (ex, eb) = EmptyBox::take(expr);
    // TODO: recursive -> iterative for trivial cases
    let result = match ex {
        // Irreducable
        Variable(_) => ex,
        Name(_) => ex, // TODO: name binding and resolution

        // Eta reduction:
        //   Reduce(\a.Ea)
        //   a is not free in E
        //=> Reduce(E)
        Abstr(var, box Appl(rest, box Variable(last)))
            if var == last && !unbounds_in(&rest).contains(&var) =>
        {
            st.etas += 1;
            st.reduced = true;
            *do_reduce(rest, st)
        }
        //   Reduce[\a.E]  =>  \a.Reduce[E]
        Abstr(var, body) => {
            let e = do_reduce(body, st);
            Abstr(var, e)
        }

        // Beta reduction:
        //   Reduce[(\x.A)B]  => Reduce[A[x->B]]
        Appl(box Abstr(from, body), to) => {
            st.betas += 1;
            st.reduced = true;
            *beta_reduce(body, from, to)
        }
        //   Reduce[AB]
        Appl(a, to) => {
            let red_box = do_reduce(a, st);
            let (reduced_a, red_eb) = EmptyBox::take(red_box);
            match reduced_a {
                //   if Reduce[A] => \x.C
                //        Beta reduction:
                //        Reduce[AB] => Reduce[(\x.C)B] => Reduce[C[x->B]]
                Abstr(from, body) => {
                    st.betas += 1;
                    st.reduced = true;
                    *beta_reduce(body, from, to)
                }
                //   else Reduce[AB] => (Reduce[A])(Reduce[B])
                other => {
                    let e = do_reduce(to, st);
                    Appl(red_eb.put(other), e)
                }
            }
        }
    };
    eb.put(result)
}

fn beta_reduce(expr: Box<Expr>, from: u8, to: Box<Expr>) -> Box<Expr> {
    let mut unbounds_to = unbounds_in(&to);
    unbounds_to.insert(from);

    //println!("beta_reduce: {}", expr);
    //println!("  from: {}", ascii::escape_default(from).to_string());
    //println!("  to  : {}", to);

    //print!("  unbound in to: ");
    //for &a in unbounds_to.iter() {
    //    print!("{}", ascii::escape_default(a).to_string())
    //}
    //println!();

    fn beta(
        expr: Box<Expr>,
        rest @ (from, to, to_unb): &(u8, Box<Expr>, HashSet<u8>),
    ) -> Box<Expr> {
        use Expr::*;
        let (ex, eb) = EmptyBox::take(expr);
        eb.put(match ex {
            Name(_) => ex,
            Appl(a, b) => Appl(beta(a, rest), beta(b, rest)),
            Abstr(v, b) => {
                if v == *from {
                    Abstr(v, b)
                } else if to_unb.contains(&v) {
                    let (v, e) = alpha(v, b, to_unb);
                    Abstr(v, beta(e, rest))
                } else {
                    Abstr(v, beta(b, rest))
                }
            }
            Variable(v) => {
                if v == *from {
                    *to.clone()
                } else {
                    ex
                }
            }
        })
    }

    beta(expr, &(from, to, unbounds_to))
}

fn alpha_next(taken: &HashSet<u8>) -> u8 {
    for letter in b'a'..=b'z' {
        if !taken.contains(&letter) {
            return letter;
        }
    }
    panic!("Ran out of variables");
}

fn alpha(par: u8, body: Box<Expr>, taken: &HashSet<u8>) -> (u8, Box<Expr>) {
    let unused = alpha_next(taken);
    (unused, replace_var(body, par, unused))
}

fn replace_var(expr: Box<Expr>, from: u8, to: u8) -> Box<Expr> {
    use Expr::*;
    let (ex, eb) = EmptyBox::take(expr);
    eb.put(match ex {
        Name(_) => ex,
        Appl(a, b) => Appl(replace_var(a, from, to), replace_var(b, from, to)),
        Abstr(v, b) => {
            if v == from {
                Abstr(v, b)
            } else {
                Abstr(v, replace_var(b, from, to))
            }
        }
        Variable(v) => {
            if v == from {
                Variable(to)
            } else {
                ex
            }
        }
    })
}
