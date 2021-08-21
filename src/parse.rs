use std::error;
use std::error::Error;
use std::fmt;
use std::iter::Peekable;
use std::vec;

use crate::expr::Expr;
use crate::lex::*;

#[derive(Clone, Debug)]
enum Atom {
    E(Box<Expr>),
    AbstrParam(u8),
    ParenStart,
}

#[derive(Clone)]
pub struct ParseError {
    pub msg: String,
    pub row: u32,
    pub col: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ParserState {
    InExpr,
    AbstrInit,
    AbstrParams,
}

type TokPeekable = Peekable<vec::IntoIter<TokenPos>>;

pub fn parse(tokps: Vec<TokenPos>) -> Result<Box<Expr>, Box<dyn Error>> {
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

fn parse_pkbl(pkbl: &mut TokPeekable) -> Result<Box<Expr>, Box<dyn Error>> {
    use Atom::*;
    use Expr::*;
    use ParserState::*;

    let mut state = InExpr;
    let mut stack: Vec<Atom> = vec![];

    let mut gcol: u32 = 0;
    let mut grow: u32 = 0;

    while let Some(tokp) = pkbl.next() {
        use crate::lex::Token::*;

        let TokenPos { tok, row, col } = tokp;
        gcol = col;
        grow = row;
        match (&state, tok) {
            (InExpr, Char(v)) => {
                append(&mut stack, Box::new(Variable(v)));
            }
            (InExpr, Capitalized(s)) => {
                append(&mut stack, Box::new(Name(s)));
            }
            (InExpr, OpParen) => {
                stack.push(ParenStart);
            }
            (InExpr, ClParen) => {
                let mut top =
                    match stack.pop() {
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
                        None => return Err(ParseError::boxed(
                            "First token should not be a closing parenthesis for spiritual reasons",
                            row,
                            col,
                        )),
                        Some(E(expr)) => expr,
                    };
                loop {
                    // until top atom isn't the start of a paren pair
                    // ..or nothing is left
                    top = match stack.pop() {
                        Some(ParenStart) => break,
                        Some(AbstrParam(p)) => Box::new(Abstr(p, top)),
                        Some(E(expr)) => Box::new(Appl(expr, top)),
                        None => {
                            return Err(ParseError::boxed(
                                "Closing parenthesis has no opening parenthesis",
                                row,
                                col,
                            ))
                        }
                    }
                }
                append(&mut stack, top);
            }
            (InExpr, Backslash) => {
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
            (s, t) => {
                return Err(ParseError::boxed(
                    format!("Unexpected token: {:?} while in state {:?}", t, s,),
                    row,
                    col,
                ))
            }
        }
    }
    if state == InExpr {
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
            None => return Err(ParseError::boxed("Can't parse nothing", grow, gcol)),
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
                None => break
            }
        }
        Ok(top)
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
    use crate::lex::lex;
    use Expr::*;

    #[test]
    fn parse1() {
        let p1 = lex("asd".as_bytes()).and_then(parse).unwrap();
        assert!(matches!(
            p1,
            box Appl(
                box Appl(box Variable(b'a'), box Variable(b's')),
                box Variable(b'd')
            )
        ));
        let p2 = lex("(as)d".as_bytes()).and_then(parse).unwrap();
        assert!(p1.alpha_eq(&p2));
        let p3 = lex("a(sd)".as_bytes()).and_then(parse).unwrap();
        assert!(!p1.alpha_eq(&p3));

        let p = lex("Name".as_bytes()).and_then(parse).unwrap();
        assert!(matches!(
           p, box Name(n) if n == "Name"
        ));
        let p = lex("1234".as_bytes()).and_then(parse).unwrap();
        assert!(matches!(
           p, box Name(n) if n == "1234"
        ));
        let p = lex("_23asdf_dfs".as_bytes()).and_then(parse).unwrap();
        assert!(matches!(
           p, box Name(n) if n == "_23asdf_dfs"
        ));

        assert!(lex(r"(as)dasfd".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"(as)dasfd".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"ASDFfdas".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"(asadf)(asdf)".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"\asdf.pbj".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"\asdf.(pb)j".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"(asa\df.fovp)(asdf)".as_bytes())
            .and_then(parse)
            .is_ok());
        assert!(lex(r"(asa)asdf(asdf)".as_bytes()).and_then(parse).is_ok());
        assert!(lex(r"asa)".as_bytes()).and_then(parse).is_err());
        assert!(lex(r"(asa\df.)(asdf)".as_bytes()).and_then(parse).is_err());
        assert!(lex(r"(asa\.asdf)(asdf)".as_bytes())
            .and_then(parse)
            .is_err());
        assert!(lex(r"(asa\()).asdf)(asdf)".as_bytes())
            .and_then(parse)
            .is_err());
        assert!(lex(r"(asa\()(.asdf)(asdf)".as_bytes())
            .and_then(parse)
            .is_err());
        assert!(lex(r"()asdf(asdf)".as_bytes()).and_then(parse).is_err());
        assert!(lex(r"asa)asdf(asdf".as_bytes()).and_then(parse).is_err());
    }
}
