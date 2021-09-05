use crate::expr::Expr;
use empty_box::EmptyBox;
use std::collections::HashSet;
use std::mem;
use std::ptr;

pub fn beta_reduce(expr: Box<Expr>, from: u8, to: Box<Expr>) -> Box<Expr> {
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
                        // Safety:
                        // Cto is never null. It always gets assigned a valid reference
                        // It doesn't get invalidated, because everything is mutated once only
                        Clone(cto) => unsafe { eb.put((**cto).clone()) },
                        Move(_) => {
                            let oto = mem::replace(to, Clone(ptr::null()));
                            if let Move(a) = oto {
                                *to = Clone(a.as_ref() as *const Expr); // Here
                                a
                            } else {
                                unreachable!()
                            }
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

pub fn alpha_next(taken: &HashSet<u8>) -> u8 {
    for letter in b'a'..=b'z' {
        if !taken.contains(&letter) {
            return letter;
        }
    }
    panic!("Ran out of variables");
}

pub fn alpha(par: u8, body: Box<Expr>, to_taken: &HashSet<u8>) -> (u8, Box<Expr>) {
    let mut taken = body.unbounds();
    taken.extend(to_taken.iter());
    let unused = alpha_next(&taken);
    (unused, replace_var(body, par, unused))
}

pub fn replace_var(expr: Box<Expr>, from: u8, to: u8) -> Box<Expr> {
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
