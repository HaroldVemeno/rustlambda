use std::{ascii, fmt};

#[derive(Clone, Debug)]
pub enum Expr {
    Variable(u8),
    Name(String),
    Abstr(u8, Box<Expr>),
    Appl(Box<Expr>, Box<Expr>),
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
