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

pub type Defs = HashMap<String, Def>;
pub struct Def {
    pub value: Box<Expr>,
}

#[macro_use]
pub mod expr_aliases {
    use super::*;

    pub fn var(v: u8) -> Box<Expr> {
        Box::new(Expr::Variable(v))
    }
    pub fn name(s: impl Into<String>) -> Box<Expr> {
        Box::new(Expr::Name(s.into()))
    }
    pub fn abstr(p: u8, e: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Abstr(p, e))
    }
    pub fn appl(f: Box<Expr>, x: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Appl(f, x))
    }

    pub fn chnum(n: u32) -> Box<Expr> {
        Expr::church_num(n)
    }

    #[macro_export]
    macro_rules! vappl {
        ($f:expr, $x:expr) => {
            appl($f, $x)
        };
        ($f:expr, $x:expr, $($rest:expr),+) => {
            vappl!(appl($f, $x), $($rest),+)
        };
        ($f:expr, $x:expr, $($rest:expr,)+) => {
            vappl!(appl($f, $x), $($rest),+)
        };
    }

    #[macro_export]
    macro_rules! vabstr {
        ($x:expr) => {
            $x
        };
        ($p:expr, $($rest:expr),+) => {
            abstr($p, vabstr!($($rest),+))
        };
        ($p:expr, $($rest:expr,)+) => {
            abstr($p, vabstr!($($rest),+))
        };
    }
}

impl Expr {
    pub fn alpha_eq(&self, other: &Self) -> bool {
        use Expr::*;
        match (self, other) {
            (Variable(v), Variable(w)) => v == w,
            (Name(n), Name(m)) => n == m,
            (Abstr(a_var, box a_body), Abstr(b_var, box b_body)) => {
                if a_var == b_var {
                    a_body.alpha_eq(b_body)
                } else {
                    let mut map = HashMap::new();
                    map.insert(*b_var, *a_var);
                    Expr::alpha_eq_mapped(a_body, b_body, map)
                }
            }
            (Appl(a_f, a_x), Appl(b_f, b_x)) => a_f.alpha_eq(b_f) && a_x.alpha_eq(b_x),
            _ => false,
        }
    }

    fn alpha_eq_mapped(&self, other: &Self, mut other_to_self: HashMap<u8, u8>) -> bool {
        use Expr::*;
        let conv = |c| other_to_self.get(c).unwrap_or(c);
        match (self, other) {
            (Variable(v), Expr::Variable(w)) => v == conv(w),
            (Name(n), Expr::Name(m)) => n == m,
            (Abstr(v, box b), Expr::Abstr(w, box c)) => {
                if v != w {
                    other_to_self.insert(*w, *v);
                }
                Expr::alpha_eq_mapped(b, c, other_to_self)
            }
            (Appl(f, x), Expr::Appl(g, y)) => {
                Expr::alpha_eq_mapped(f, g, other_to_self.clone())
                    && Expr::alpha_eq_mapped(x, y, other_to_self)
            }
            _ => false,
        }
    }

    pub fn size(&self) -> u32 {
        use Expr::*;
        let mut ret = 0;
        let mut stack = Vec::new();
        let mut next = self;
        loop {
            ret += 1;
            next = match next {
                Variable(_) | Name(_) => match stack.pop() {
                    Some(e) => e,
                    None => break,
                },
                Abstr(_, body) => body,
                Appl(a, b) => {
                    ret -= 1;
                    stack.push(a);
                    b
                }
            }
        }
        ret
    }

    pub fn unbounds(&self) -> HashSet<u8> {
        use Expr::*;
        match self {
            Variable(v) => {
                let mut set = HashSet::new();
                set.insert(*v);
                set
            }
            Name(_) => HashSet::new(),
            Abstr(v, b) => {
                let mut set = b.unbounds();
                set.remove(v);
                set
            }
            Appl(a, b) => {
                let mut set = a.unbounds();
                for item in b.unbounds() {
                    set.insert(item);
                }
                set
            }
        }
    }

    pub fn church_num(mut n: u32) -> Box<Expr> {
        use Expr::*;
        let mut ret = Box::new(Variable(b'x'));
        while n > 0 {
            ret = Box::new(Appl(Box::new(Variable(b'f')), ret));
            n -= 1;
        }
        ret = Box::new(Abstr(b'f', Box::new(Abstr(b'x', ret))));
        ret
    }

    pub fn try_unchurch_num(&self) -> Option<u32> {
        use Expr::*;
        match self {
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
    use super::expr_aliases::*;
    use super::*;
    use crate::test::*;
    use std::error::Error;
    use Expr::*;

    #[test]
    fn unbounds_in_0() -> Result<(), Box<dyn Error>> {
        let mut set = process(r#"\afc.fpad(\qc.cag)"#).unbounds();
        assert_eq!(set.len(), 3);
        assert!(set.remove(&b'p') && set.remove(&b'd') && set.remove(&b'g'));
        assert!(set.is_empty());
        Ok(())
    }

    #[test]
    fn alpha_eq_0() -> Result<(), Box<dyn Error>> {
        let e1 = process(r#"\fad.da(lf)"#);
        let e2 = process(r#"\okg.gk(lo)"#);
        assert!(e1.alpha_eq(&e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_1() -> Result<(), Box<dyn Error>> {
        let e1 = process(r#"\abc.ba(\b.cb)(\ac.ba)(\ap.caa)"#);
        let e2 = process(r#"\bap.ab(\a.ca)(\bc.ab)(\vc.pvv)"#);
        assert!(e1.alpha_eq(&e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_2() -> Result<(), Box<dyn Error>> {
        let e1 = process(r#"\abg.ba(\b.cb)(\ac.ba)(\ap.caa)"#);
        let e2 = process(r#"\bap.ab(\a.ca)(\bc.ab)(\vc.pvv)"#);
        assert!(!e1.alpha_eq(&e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_3() -> Result<(), Box<dyn Error>> {
        let e1 = process(r#"\abc.ba(\b.cb)(\ac.ab)(\ap.caa)"#);
        let e2 = process(r#"\bap.ab(\a.ca)(\bc.ab)(\vc.pvv)"#);
        println!("{}", &e1);
        println!("{}", &e2);
        assert!(!e1.alpha_eq(&e2));
        Ok(())
    }

    #[test]
    fn alpha_eq_4() -> Result<(), Box<dyn Error>> {
        let e1 = process(r#"\abc.ba(\b.cb)(\ac.ba)(\ap.caa)"#);
        let e2 = process(r#"\bap.ab(\a.ca)(\bc.ab)(\va.pvv)"#);
        assert!(e1.alpha_eq(&e2));
        Ok(())
    }

    #[test]
    fn church_nums_0() {
        let zero = Expr::church_num(0);
        assert!(zero.alpha_eq(&Abstr(b'd', abstr(b'r', var(b'r')))));
        assert_eq!(zero.try_unchurch_num(), Some(0));
    }

    #[test]
    fn church_nums_1() {
        let one = Expr::church_num(1);
        assert!(one.alpha_eq(&Abstr(b'd', abstr(b'r', appl(var(b'd'), var(b'r'))))));
        assert_eq!(one.try_unchurch_num(), Some(1));
    }

    #[test]
    fn church_nums_7() {
        assert_eq!(Expr::church_num(7).try_unchurch_num(), Some(7));
    }
}
