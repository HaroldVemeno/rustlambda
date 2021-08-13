use std::{
    ascii,
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Clone, Debug)]
pub enum Expr {
    Variable(u8),
    Name(String),
    Abstr(u8, Box<Expr>),
    Appl(Box<Expr>, Box<Expr>),
}

pub fn alpha_eq(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::Variable(v), Expr::Variable(w)) => v == w,
        (Expr::Name(n), Expr::Name(m)) => n == m,
        (Expr::Abstr(v, box b), Expr::Abstr(w, box c)) => {
            if v == w {
                alpha_eq(b, c)
            } else {
                let mut map = HashMap::new();
                map.insert(*w, *v);
                alpha_eq_mapped(b, c, map)
            }
        }
        (Expr::Appl(f, x), Expr::Appl(g, y)) => alpha_eq(f, g) && alpha_eq(x, y),
        _ => false,
    }
}

pub fn alpha_eq_mapped(a: &Expr, b: &Expr, mut b_to_a: HashMap<u8, u8>) -> bool {
    let conv = |c| b_to_a.get(c).unwrap_or(c);
    match (a, b) {
        (Expr::Variable(v), Expr::Variable(w)) => v == conv(w),
        (Expr::Name(n), Expr::Name(m)) => n == m,
        (Expr::Abstr(v, box b), Expr::Abstr(w, box c)) => {
            if v == w {
                alpha_eq_mapped(b, c, b_to_a)
            } else {
                b_to_a.insert(*w, *v);
                alpha_eq_mapped(b, c, b_to_a)
            }
        }
        (Expr::Appl(f, x), Expr::Appl(g, y)) => {
            alpha_eq_mapped(f, g, b_to_a.clone()) && alpha_eq_mapped(x, y, b_to_a)
        }
        _ => false,
    }
}

pub fn size(expr: &Expr) -> u32 {
    let mut ret = 0;
    let mut stack = Vec::new();
    let mut next = expr;
    loop {
        ret += 1;
        next = match next {
            Expr::Variable(_) | Expr::Name(_) => match stack.pop() {
                Some(e) => e,
                None => break,
            },
            Expr::Abstr(_, body) => body,
            Expr::Appl(a, b) => {
                stack.push(a);
                b
            }
        }
    }
    ret
}

pub fn unbounds_in(expr: &Expr) -> HashSet<u8> {
    match expr {
        Expr::Variable(v) => {
            let mut set = HashSet::new();
            set.insert(*v);
            set
        }
        Expr::Name(_) => HashSet::new(),
        Expr::Abstr(v, b) => {
            let mut set = unbounds_in(b);
            set.remove(v);
            set
        }
        Expr::Appl(a, b) => {
            let mut set = unbounds_in(a);
            for item in unbounds_in(b) {
                set.insert(item);
            }
            set
        }
    }
}

pub fn to_church_num(mut n: u32) -> Box<Expr> {
    use Expr::*;
    let mut ret = Box::new(Variable(b'x'));
    while n > 0 {
        ret = Box::new(Appl(Box::new(Variable(b'f')), ret));
        n -= 1;
    }
    ret = Box::new(Abstr(b'f', Box::new(Abstr(b'x', ret))));
    ret
}

pub fn from_church_num(e: &Expr) -> Option<u32> {
    use Expr::*;
    match e {
        Abstr(f, box Abstr(x, box body)) => {
            let mut ret: u32 = 0;
            let mut b: &Expr = body;
            loop {
                b = match b {
                    Variable(v) if v == x => break Some(ret),
                    Appl(box Variable(g), box y) if g == f => {
                        ret += 1;
                        y
                    }
                    _ => break None,
                }
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Expr::*;

    #[test]
    fn church_nums0() {
        let zero = to_church_num(0);
        assert!(alpha_eq(
            zero.as_ref(),
            &Abstr(b'd', Box::new(Abstr(b'r', Box::new(Variable(b'r')))))
        ));
        assert_eq!(from_church_num(zero.as_ref()), Some(0));
    }

    #[test]
    fn church_nums1() {
        let one = to_church_num(1);
        assert!(alpha_eq(
            one.as_ref(),
            &Abstr(
                b'd',
                Box::new(Abstr(
                    b'r',
                    Box::new(Appl(Box::new(Variable(b'd')), Box::new(Variable(b'r'))))
                ))
            )
        ));
        assert_eq!(from_church_num(one.as_ref()), Some(1));
    }

    #[test]
    fn church_nums7() {
        assert_eq!(from_church_num(to_church_num(7).as_ref()), Some(7));
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        if !f.alternate() {
            // Valid lambda expression
            match self {
                Variable(v) => write!(f, "{}", ascii::escape_default(*v)),
                Name(n) => write!(f, " {} ", n),
                Abstr(p, b) => {
                    let mut body = b.as_ref();
                    write!(f, "\\{}", ascii::escape_default(*p))?;
                    while let Abstr(pn, bn) = body {
                        write!(f, "{}", ascii::escape_default(*pn))?;
                        body = bn.as_ref();
                    }
                    write!(f, ".{}", body)
                }
                Appl(a, b) => match (a.as_ref(), b.as_ref()) {
                    (Abstr(_, _), Appl(_, _) | Abstr(_, _)) => write!(f, "({})({})", a, b),
                    (Abstr(_, _), _) => write!(f, "({}){}", a, b),
                    (_, Appl(_, _) | Abstr(_, _)) => write!(f, "{}({})", a, b),
                    _ => write!(f, "{}{}", a, b),
                },
            }
        } else {
            // A tree representing the expression
            fn tree(
                f: &mut fmt::Formatter<'_>,
                head_prepend: String,
                mut rest_prepend: String,
                expr: &Expr,
            ) -> fmt::Result {
                match expr {
                    Variable(v) => writeln!(f, "{}Var {}", head_prepend, ascii::escape_default(*v)),
                    Name(n) => writeln!(f, "{}Name {}", head_prepend, n),
                    Abstr(p, b) => {
                        write!(f, "{}Abstr", head_prepend)?;
                        let mut body = b.as_ref();
                        write!(f, " {}", ascii::escape_default(*p))?;
                        while let Abstr(pn, bn) = body {
                            write!(f, " {}", ascii::escape_default(*pn))?;
                            body = bn.as_ref();
                        }
                        writeln!(f)?;
                        let rest = rest_prepend.clone() + "  ";
                        rest_prepend.push_str("`-");
                        tree(f, rest_prepend, rest, body)
                    }
                    Appl(a, b) => {
                        writeln!(f, "{}.", head_prepend)?;
                        let mut stack: Vec<&Expr> = vec![b];
                        let mut current_a = a.as_ref();
                        while let Appl(aa, ab) = current_a {
                            stack.push(ab.as_ref());
                            current_a = aa.as_ref();
                        }
                        stack.push(current_a);

                        let head = rest_prepend.clone() + "|-";
                        let rest = rest_prepend.clone() + "| ";
                        while stack.len() > 1 {
                            tree(f, head.clone(), rest.clone(), stack.pop().unwrap())?;
                        }

                        let last_rest = rest_prepend.clone() + "  ";
                        rest_prepend.push_str("`-");
                        tree(f, rest_prepend, last_rest, stack.pop().unwrap())
                    }
                }
            }
            tree(f, String::new(), String::new(), self)
        }
    }
}
