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

pub fn alpha_eq(a_in: &Expr, b_in: &Expr) -> bool {
    match (a_in, b_in) {
        (Expr::Variable(v), Expr::Variable(w)) => v == w,
        (Expr::Name(n), Expr::Name(m)) => n == m,
        (Expr::Abstr(a_var, box a_body), Expr::Abstr(b_var, box b_body)) => {
            if a_var == b_var {
                alpha_eq(a_body, b_body)
            } else {
                let mut map = HashMap::new();
                map.insert(*b_var, *a_var);
                alpha_eq_mapped(a_body, b_body, map)
            }
        }
        (Expr::Appl(a_f, a_x), Expr::Appl(b_f, b_x)) => alpha_eq(a_f, b_f) && alpha_eq(a_x, b_x),
        _ => false,
    }
}

pub fn alpha_eq_mapped(a: &Expr, b: &Expr, mut b_to_a: HashMap<u8, u8>) -> bool {
    let conv = |c| b_to_a.get(c).unwrap_or(c);
    match (a, b) {
        (Expr::Variable(v), Expr::Variable(w)) => v == conv(w),
        (Expr::Name(n), Expr::Name(m)) => n == m,
        (Expr::Abstr(v, box b), Expr::Abstr(w, box c)) => {
            if v != w {
                b_to_a.insert(*w, *v);
            }
            alpha_eq_mapped(b, c, b_to_a)
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

///Expr should be fully reduced!!
pub fn from_church_num(ch_num: &Expr) -> Option<u32> {
    use Expr::*;
    match ch_num {
        Abstr(bound_f, box Abstr(bound_x, box body)) => {
            let mut ret: u32 = 0;
            let mut current_body: &Expr = body;
            loop {
                current_body = match current_body {
                    Variable(now_x) if now_x == bound_x => break Some(ret),
                    Appl(box Variable(now_f), box rest) if now_f == bound_f => {
                        ret += 1;
                        rest
                    }
                    _ => break None,
                }
            }
        }
        _ => None,
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::{parse::parse, lex::lex};

    use super::*;
    use Expr::*;

    #[test]
    fn unbounds_in_0() -> Result<(), Box<dyn Error>> {
        let mut set = unbounds_in(parse(lex(r#"\afc.fpad(\qc.cag)"#.as_bytes())?)?.as_ref());
        assert_eq!(set.len(), 3);
        assert!(set.remove(&b'p') && set.remove(&b'd') && set.remove(&b'g'));
        assert!(set.is_empty());
        Ok(())

    }

    #[test]
    fn alpha_eq_0() -> Result<(),Box<dyn Error>> {
        let e1 = parse(lex(r#"\fad.da(lf)"#.as_bytes())?)?;
        let e2 = parse(lex(r#"\okg.gk(lo)"#.as_bytes())?)?;
        assert!(alpha_eq(&e1, &e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_1() -> Result<(),Box<dyn Error>> {
        let e1 = parse(lex(r#"\abc.ba(\b.cb)(\ac.ba)(\ap.caa)"#.as_bytes())?)?;
        let e2 = parse(lex(r#"\bap.ab(\a.ca)(\bc.ab)(\vc.pvv)"#.as_bytes())?)?;
        assert!(alpha_eq(&e1, &e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_2() -> Result<(),Box<dyn Error>> {
        let e1 = parse(lex(r#"\abg.ba(\b.cb)(\ac.ba)(\ap.caa)"#.as_bytes())?)?;
        let e2 = parse(lex(r#"\bap.ab(\a.ca)(\bc.ab)(\vc.pvv)"#.as_bytes())?)?;
        assert!(!alpha_eq(&e1, &e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_3() -> Result<(),Box<dyn Error>> {
        let e1 = parse(lex(r#"\abc.ba(\b.cb)(\ac.ab)(\ap.caa)"#.as_bytes())?)?;
        let e2 = parse(lex(r#"\bap.ab(\a.ca)(\bc.ab)(\vc.pvv)"#.as_bytes())?)?;
        assert!(!alpha_eq(&e1, &e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_4() -> Result<(),Box<dyn Error>> {
        let e1 = parse(lex(r#"\abc.ba(\b.cb)(\ac.ba)(\ap.caa)"#.as_bytes())?)?;
        let e2 = parse(lex(r#"\bap.ab(\a.ca)(\bc.ab)(\va.pvv)"#.as_bytes())?)?;
        assert!(alpha_eq(&e1, &e2));
        Ok(())
    }

    #[test]
    fn church_nums_0() {
        let zero = to_church_num(0);
        assert!(alpha_eq(
            zero.as_ref(),
            &Abstr(b'd', Box::new(Abstr(b'r', Box::new(Variable(b'r')))))
        ));
        assert_eq!(from_church_num(zero.as_ref()), Some(0));
    }

    #[test]
    fn church_nums_1() {
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
    fn church_nums_7() {
        assert_eq!(from_church_num(to_church_num(7).as_ref()), Some(7));
    }
}

