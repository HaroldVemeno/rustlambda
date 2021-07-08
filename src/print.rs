use std::{ascii, error, fmt};

use crate::lex;
use crate::parse;

impl error::Error for lex::LexError {}

impl fmt::Display for lex::LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lex::LexError { row, col, msg } = self;
        write!(f, "LexError({}:{}): {}", row, col, msg)
    }
}

impl fmt::Debug for lex::LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lex::LexError { row, col, msg } = self;
        write!(f, "LexError({}:{}): {}", row, col, msg)
    }
}

impl error::Error for parse::ParseError {}

impl fmt::Display for parse::ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parse::ParseError { row, col, msg } = self;
        write!(f, "ParseError({}:{}): {}", row, col, msg)
    }
}

impl fmt::Debug for parse::ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parse::ParseError { row, col, msg } = self;
        write!(f, "ParseError({}:{}): {}", row, col, msg)
    }
}

impl fmt::Display for lex::Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use lex::Token::*;
        match &self {
            Char(c) => f.write_str(&ascii::escape_default(*c).to_string()),
            Capitalized(s) => f.write_str(s),
            Backslash => write!(f, "\\"),
            OpParen => write!(f, "("),
            ClParen => write!(f, ")"),
            Dot => write!(f, "."),
        }
    }
}

impl fmt::Display for lex::TokenPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tok.fmt(f)
    }
}

impl fmt::Display for parse::Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use parse::Expr::{self, *};
        if !f.alternate() {
            match self {
                Variable(v) => write!(f, "{}", ascii::escape_default(*v)),
                Name(n) => write!(f, "{}", n),
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
                        let mut stack: Vec<&Expr> = vec!(b);
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
