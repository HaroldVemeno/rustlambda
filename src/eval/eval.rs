use empty_box::EmptyBox;
use std::collections::HashSet;
use std::error;
use std::fmt;
use std::mem;
use std::panic;

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
        let expr_size = expr.size();
        debug_assert_eq!(expr_size, stats.size);
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

fn beta_reduce(expr: Box<Expr>, from: u8, to: Box<Expr>) -> Box<Expr> {
    let mut unbounds_to = to.unbounds();
    unbounds_to.insert(from);

    enum Linear {
        Move(Box<Expr>),
        Clone(*const Expr),
    }
    use Linear::*;

    //println!("beta_reduce: {}", expr);
    //println!("  from: {}", ascii::escape_default(from).to_string());
    //println!("  to  : {}", to);

    //print!("  unbound in to: ");
    //for &a in unbounds_to.iter() {
    //    print!("{}", ascii::escape_default(a).to_string())
    //}
    //println!();

    fn beta(expr: Box<Expr>, from: u8, to: &mut Linear, to_unb: &HashSet<u8>) -> Box<Expr> {
        use Expr::*;
        let (ex, eb) = EmptyBox::take(expr);
        match ex {
            Name(_) => eb.put(ex),
            Appl(a, b) => eb.put(Appl(beta(a, from, to, to_unb), beta(b, from, to, to_unb))),
            Abstr(v, b) => eb.put(if v == from {
                Abstr(v, b)
            } else if to_unb.contains(&v) {
                let (v, e) = alpha(v, b, to_unb);
                Abstr(v, beta(e, from, to, to_unb))
            } else {
                Abstr(v, beta(b, from, to, to_unb))
            }),
            Variable(v) => {
                if v == from {
                    match to {
                        Clone(cto) => unsafe { eb.put((**cto).clone()) },
                        Move(_) => {
                            let oto = mem::replace(to, Clone(0 as *const Expr));
                            if let Move(a) = oto {
                                *to = Clone(a.as_ref() as *const Expr);
                                a
                            } else {unreachable!()}
                        }
                    }
                } else {
                    eb.put(ex)
                }
            }
        }

    }

    beta(expr, from, &mut Move(to), &unbounds_to)
}

fn alpha_next(taken: &HashSet<u8>) -> u8 {
    for letter in b'a'..=b'z' {
        if !taken.contains(&letter) {
            return letter;
        }
    }
    panic!("Ran out of variables");
}

fn alpha(par: u8, body: Box<Expr>, to_taken: &HashSet<u8>) -> (u8, Box<Expr>) {
    let mut taken = body.unbounds();
    taken.extend(to_taken.iter());
    let unused = alpha_next(&taken);
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
