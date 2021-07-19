use std::collections::HashSet;
use std::error::Error;
use empty_box::EmptyBox;

use crate::types::Expr;

pub fn reduce_nolog(mut expr: Box<Expr>) -> Result<Box<Expr>, Box<dyn Error>> {
    let max_repetitions = 10000;
    let max_size = 10000;

    for _ in 1..max_repetitions {
        expr = do_reduce_nolog(expr);
        // TODO: actual checks and stuff
    }
    Ok(expr)
}

fn do_reduce_nolog(expr: Box<Expr>) -> Box<Expr> {
    use Expr::*;
    let (ex, eb) = EmptyBox::take(expr);
    eb.put(match ex {
        Variable(_) | Name(_) => ex,
        Abstr(var, box Appl(rest, box Variable(last))) if var == last => {
            *do_reduce_nolog(rest)
        }
        Abstr(var, body) => {
            Abstr(var, do_reduce_nolog(body))
        }
        Appl(box Abstr(from, body), to) => {
            *do_reduce_nolog(beta_reduce(body, from, to))
        }
        Appl(a, to) => {
            let (reduced_a, red_box) = EmptyBox::take(do_reduce_nolog(a));
            match reduced_a {
                Abstr(from, body) => 
                    *do_reduce_nolog(beta_reduce(body, from, to)),
                other => Appl(red_box.put(other), do_reduce_nolog(to))
            }
        }
    })
}

fn beta_reduce(expr: Box<Expr>, from: u8, to: Box<Expr>) -> Box<Expr> {
    let bound_in_expr = free_variables(&expr);
    let bound_in_to = free_variables(&to);

    unimplemented!("beta_reduce is not implemented yet")
}

fn free_variables(expr: &Expr) -> HashSet<u8> {
    match expr {
        Expr::Variable(v) => {
            let mut set = HashSet::new();
            set.insert(*v);
            set
        }
        Expr::Name(_) => HashSet::new(),
        Expr::Abstr(v, b) => {
            let mut set = free_variables(b);
            set.remove(v);
            set
        }
        Expr::Appl(a, b) => {
            let mut set = free_variables(a);
            for item in free_variables(b) {
                set.insert(item);
            }
            set
        }
    }   
}
