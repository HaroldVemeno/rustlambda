use core::panic;
use empty_box::EmptyBox;
use std::collections::HashSet;
use std::{error, fmt};

use crate::expr::Expr;

#[derive(Clone)]
pub struct EvalError {
    msg: String,
}

pub fn reduce_nolog(mut expr: Box<Expr>) -> Result<Box<Expr>, Box<dyn error::Error>> {
    let max_iterations = 1000;
    let step = 1;
    let max_size = 1000000;

    for i in (1..=max_iterations).step_by(step) {
        for s in 1..=step {
            expr = match do_reduce_nolog(expr) {
                (e, true) => e,
                (e, false) => {
                    println!("Took {} iterations", i + s);
                    return Ok(e);
                }
            };
        }
        //println!("{}", expr);
        let expr_size = size(&expr);
        if expr_size > max_size {
            return Err(EvalError::boxed(format!(
                "Size outgrew maximum size: {} out of {}",
                expr_size, max_size
            )));
        }
    }
    Err(EvalError::boxed(format!(
        "Iteration limit reached: {}",
        max_iterations
    )))
}

fn do_reduce_nolog(expr: Box<Expr>) -> (Box<Expr>, bool) {
    //println!("do_reduce_nolog: {}", expr);

    use Expr::*;
    let (ex, eb) = EmptyBox::take(expr);
    // TODO: recursive -> iterative for trivial cases
    let (result, reduced) = match ex {
        // Irreducable
        Variable(_) => (ex, false),
        Name(_) => (ex, false), // TODO: name binding and resolution

        // Eta reduction:
        //   Reduce(\a.Ea)
        //   a is not free in E
        //=> Reduce(E)
        Abstr(var, box Appl(rest, box Variable(last)))
            if var == last && !unbounds_in(&rest).contains(&var) =>
        {
            (*do_reduce_nolog(rest).0, true)
        }
        //   Reduce[\a.E]  =>  \a.Reduce[E]
        Abstr(var, body) => {
            let (e, r) = do_reduce_nolog(body);
            (Abstr(var, e), r)
        }

        // Beta reduction:
        //   Reduce[(\x.A)B]  => Reduce[A[x->B]]
        Appl(box Abstr(from, body), to) => (*beta_reduce(body, from, to), true),
        //   Reduce[AB]
        Appl(a, to) => {
            let (red_box, is_red) = do_reduce_nolog(a);
            let (reduced_a, red_eb) = EmptyBox::take(red_box);
            match reduced_a {
                //   if Reduce[A] => \x.C
                //        Beta reduction:
                //        Reduce[AB] => Reduce[(\x.C)B] => Reduce[C[x->B]]
                Abstr(from, body) => (*beta_reduce(body, from, to), true),
                //   else Reduce[AB] => (Reduce[A])(Reduce[B])
                other => {
                    let (e, r) = do_reduce_nolog(to);
                    (Appl(red_eb.put(other), e), r || is_red)
                }
            }
        }
    };
    (eb.put(result), reduced)
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



impl EvalError {
    fn boxed(msg: impl Into<String>) -> Box<EvalError> {
        Box::new(EvalError { msg: msg.into() })
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {}", self.msg)
    }
}

impl fmt::Debug for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {}", self.msg)
    }
}

impl error::Error for EvalError {}
