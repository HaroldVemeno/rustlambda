use std::collections::HashMap;
use std::error::{self, Error};
use std::fmt;
use std::iter::Peekable;
use std::vec;

use crate::expr::{Def, Defs, Expr};
use crate::lex::*;

#[derive(Clone, Debug)]
enum Atom {
    E(Box<Expr>),
    AbstrParam(u8),
    Definition(String),
    ParenStart,
}

#[derive(Clone)]
pub struct ParseError {
    pub msg: String,
    pub row: u32,
    pub col: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    Start,
    InExpr,
    AbstrInit,
    AbstrParams,
}

type TokPeekable = Peekable<vec::IntoIter<TokenPos>>;

pub fn parse(tokps: Vec<TokenPos>) -> Result<(Defs, Option<Box<Expr>>), Box<dyn Error>> {
    parse_pkbl(&mut tokps.into_iter().peekable())
}

fn append(stack: &mut Vec<Atom>, expr: Box<Expr>) {
    use Atom::*;
    use Expr::*;
    if matches!(stack.last(), Some(E(_))) {
        if let Some(E(before)) = stack.pop() {
            stack.push(E(Box::new(Appl(before, expr))))
        }
    } else {
        stack.push(E(expr));
    }
}

fn parse_pkbl(pkbl: &mut TokPeekable) -> Result<(Defs, Option<Box<Expr>>), Box<dyn Error>> {
    use Atom::*;
    use Expr::*;
    use State::*;

    let mut state = Start;
    let mut stack: Vec<Atom> = Vec::new();
    let mut defs: Defs = HashMap::new();

    let mut gcol: u32 = 0;
    let mut grow: u32 = 0;

    while let Some(tokp) = pkbl.next() {
        use crate::lex::Token::*;

        // dbg!(&stack);
        //dbg!(&state);

        let TokenPos { tok, row, col } = tokp;
        gcol = col;
        grow = row;
        match (&state, tok) {
            (InExpr | Start, Char(v)) => {
                append(&mut stack, Box::new(Variable(v)));
                state = InExpr;
            }
            (Start, Capitalized(s)) => {
                if let Some(TokenPos { tok: Equals, .. }) = pkbl.peek() {
                    pkbl.next();
                    stack.push(Definition(s));
                } else {
                    append(&mut stack, Box::new(Name(s)));
                }
                state = InExpr;
            }
            (InExpr, Capitalized(s)) => {
                append(&mut stack, Box::new(Name(s)));
                state = InExpr;
            }
            (InExpr | Start, OpParen) => {
                stack.push(ParenStart);
                state = InExpr;
            }
            (InExpr | Start, ClParen) => {
                let mut top = match stack.pop() {
                    Some(E(expr)) => expr,
                    Some(ParenStart) => {
                        return Err(ParseError::boxed(
                            "Attempt to close an empty expression",
                            row,
                            col,
                        ))
                    }
                    Some(AbstrParam(_)) => {
                        return Err(ParseError::boxed(
                            "Attempt to close an abstraction with an empty body",
                            row,
                            col,
                        ))
                    }
                    None | Some(Definition(_)) => {
                        return Err(ParseError::boxed(
                            "Expression starts with a closing parenthesis",
                            row,
                            col,
                        ))
                    }
                };
                loop {
                    // until top atom isn't the start of a paren pair
                    // ..or nothing is left
                    top = match stack.pop() {
                        Some(ParenStart) => break,
                        Some(AbstrParam(p)) => Box::new(Abstr(p, top)),
                        Some(E(expr)) => Box::new(Appl(expr, top)),
                        Some(Definition(_)) | None => {
                            return Err(ParseError::boxed(
                                "Closing parenthesis has no opening parenthesis",
                                row,
                                col,
                            ))
                        }
                    }
                }
                append(&mut stack, top);
                state = InExpr;
            }
            (InExpr | Start, Backslash) => {
                state = AbstrInit;
            }
            (AbstrInit, Char(v)) => {
                stack.push(AbstrParam(v));
                state = AbstrParams;
            }
            (AbstrParams, Char(v)) => {
                stack.push(AbstrParam(v));
            }
            (AbstrParams, Dot) => {
                state = InExpr;
            }
            (InExpr, Semicolon) => {
                let mut top = match stack.pop() {
                    Some(ParenStart) => {
                        return Err(ParseError::boxed(
                            "Statement ended with an open parenthesis",
                            grow,
                            gcol,
                        ))
                    }
                    Some(AbstrParam(_)) => {
                        // unreachable?
                        return Err(ParseError::boxed(
                            "Statement ended with an open abstraction",
                            grow,
                            gcol,
                        ));
                    }
                    None | Some(Definition(_)) => {
                        return Err(ParseError::boxed("Empty Expression", grow, gcol))
                    }
                    Some(E(expr)) => expr,
                };
                loop {
                    // until top atom isn't the start of a paren pair
                    // ..or nothing is left
                    top = match stack.pop() {
                        Some(ParenStart) => {
                            return Err(ParseError::boxed("An unclosed parenthesis", grow, gcol))
                        }
                        Some(AbstrParam(p)) => Box::new(Abstr(p, top)),
                        Some(E(expr)) => Box::new(Appl(expr, top)),
                        Some(Definition(s)) => {
                            assert!(stack.is_empty(), "Def should be the first element");
                            defs.insert(s, Def { value: top });
                            break;
                        }
                        None => break,
                    }
                }
                stack.pop();
                state = Start;
            }
            (s, t) => {
                return Err(ParseError::boxed(
                    format!("Unexpected token: {:?} while in state {:?}", t, s,),
                    row,
                    col,
                ))
            }
        }
    }
    if let InExpr | Start = state {
        let mut top = match stack.pop() {
            Some(ParenStart) => {
                return Err(ParseError::boxed(
                    "Input ended with an open parenthesis",
                    grow,
                    gcol,
                ))
            }
            Some(AbstrParam(_)) => {
                // unreachable?
                return Err(ParseError::boxed(
                    "Input ended with an open abstraction",
                    grow,
                    gcol,
                ));
            }
            Some(Definition(_)) => return Err(ParseError::boxed("Empty definition", grow, gcol)),
            None => {
                return Ok((defs, None));
            }
            Some(E(expr)) => expr,
        };
        loop {
            // until top atom isn't the start of a paren pair
            // ..or nothing is left
            top = match stack.pop() {
                Some(ParenStart) => {
                    return Err(ParseError::boxed("An unclosed parenthesis", grow, gcol))
                }
                Some(AbstrParam(p)) => Box::new(Abstr(p, top)),
                Some(E(expr)) => Box::new(Appl(expr, top)),
                Some(Definition(s)) => {
                    assert!(stack.is_empty(), "Def should be the first element");
                    defs.insert(s, Def { value: top });
                    return Ok((defs, None));
                }
                None => break,
            }
        }
        Ok((defs, Some(top)))
    } else {
        Err(ParseError::boxed(
            "Input ended with an unfinished abstraction",
            grow,
            gcol,
        ))
    }
}

impl ParseError {
    fn boxed(msg: impl Into<String>, row: u32, col: u32) -> Box<Self> {
        Box::new(ParseError {
            msg: msg.into(),
            row,
            col,
        })
    }
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseError { row, col, msg } = self;
        write!(f, "ParseError({}:{}): {}", row, col, msg)
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseError { row, col, msg } = self;
        write!(f, "ParseError({}:{}): {}", row, col, msg)
    }
}

impl<A> From<ParseError> for Result<A, Box<dyn Error>> {
    fn from(val: ParseError) -> Self {
        Err(Box::new(val))
    }
}

impl<A> From<ParseError> for Result<A, ParseError> {
    fn from(val: ParseError) -> Self {
        Err(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;
    use Expr::*;

    #[test]
    fn parse1() {
        let p1 = process("asd");
        assert!(matches!(
            p1,
            box Appl(
                box Appl(box Variable(b'a'), box Variable(b's')),
                box Variable(b'd')
            )
        ));
        let p2 = process("(as)d");
        assert!(p1.alpha_eq(&p2));
        let p3 = process("a(sd)");
        assert!(!p1.alpha_eq(&p3));

        let p = process("Name");
        assert!(matches!(
           p, box Name(n) if n == "Name"
        ));
        let p = process("1234");
        assert!(matches!(
           p, box Name(n) if n == "1234"
        ));
        let p = process("_23asdf_dfs");
        assert!(matches!(
           p, box Name(n) if n == "_23asdf_dfs"
        ));

        let ok = [
            r"(as)dasfd",
            r"(as)dasfd",
            r"ASDFfdas",
            r"(asadf)(asdf)",
            r"\asdf.pbj",
            r"\asdf.(pb)j",
            r"(asa\df.fovp)(asdf)",
            r"(asa)asdf(asdf)",
            r"\ame.\a.che",
        ];

        assert!(ok
            .iter()
            .map(|a| a.as_bytes())
            .map(lex)
            .map(|a| a.and_then(parse))
            .all(|a| a.is_ok()));

        let err = [
            r"asa)                ",
            r")                   ",
            r"(                   ",
            r"(aaa                ",
            r"(aaa(asdf)          ",
            r"pop)                ",
            r"\                   ",
            r"\sdf                ",
            r"\sdf.               ",
            r"\Name.a             ",
            r"\am\e.a             ",
            r"(asa\df.)(asdf)     ",
            r"(asa\.asdf)(asdf)   ",
            r"(asa\()).asdf)(asdf)",
            r"(asa\()(.asdf)(asdf)",
            r"()asdf(asdf)        ",
            r"asa)asdf(asdf       ",
        ];

        assert!(err
            .iter()
            .map(|a| a.as_bytes())
            .map(lex)
            .map(|a| a.and_then(parse))
            .all(|a| a.is_err()));
    }
}
