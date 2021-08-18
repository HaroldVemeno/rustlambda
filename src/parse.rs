use std::error;
use std::error::Error;
use std::fmt;
use std::iter::Peekable;
use std::vec;

use crate::expr::Expr;
use crate::lex::*;

enum Atom {
    E(Box<Expr>),
    AbstrParam(u8),
}

#[derive(Clone)]
pub struct ParseError {
    pub msg: String,
    pub row: u32,
    pub col: u32,
}

#[derive(Copy, Clone, Debug)]
enum ParserState {
    InExpr,
    AbstrInit,
    AbstrParams,
}

pub fn parse(tokps: Vec<TokenPos>) -> Result<Box<Expr>, Box<dyn Error>> {
    parse_pkbl(&mut tokps.into_iter().peekable(), 0, 0)
}

fn parse_pkbl(
    pkbl: &mut Peekable<vec::IntoIter<TokenPos>>,
    mut gcol: u32,
    mut grow: u32,
) -> Result<Box<Expr>, Box<dyn Error>> {
    use Atom::*;
    use Expr::*;
    use ParserState::*;

    let mut state = InExpr;
    let mut stack: Vec<Atom> = vec![];

    while let Some(tokp) = pkbl.next() {
        use crate::lex::{Token::*, *};

        let TokenPos { tok, row, col } = tokp;
        gcol = col;
        grow = row;
        match (&state, tok) {
            (InExpr, Char(v)) => {
                let var = Box::new(Variable(v));
                if matches!(stack.last(), Some(E(_))) {
                    if let Some(E(before)) = stack.pop() {
                        stack.push(E(Box::new(Appl(before, var))))
                    }
                } else {
                    stack.push(E(var));
                }
            }
            (InExpr, Capitalized(s)) => {
                let name = Box::new(Name(s));
                if matches!(stack.last(), Some(E(_))) {
                    if let Some(E(before)) = stack.pop() {
                        stack.push(E(Box::new(Appl(before, name))))
                    }
                } else {
                    stack.push(E(name));
                }
            }
            (InExpr, OpParen) => {
                let scope = parse_pkbl(pkbl, gcol, grow)?;
                if matches!(stack.last(), Some(E(_))) {
                    if let Some(E(before)) = stack.pop() {
                        stack.push(E(Box::new(Appl(before, scope))))
                    }
                } else {
                    stack.push(E(scope));
                }
            }
            (InExpr, ClParen) => {
                if stack.len() == 1 {
                    if let Some(E(e)) = stack.pop() {
                        return Ok(e);
                    } else {
                        return Err(ParseError::boxed(
                            "Attempt to close an empty expression",
                            row,
                            col,
                        ));
                    }
                } else if stack.is_empty() {
                    return Err(ParseError::boxed(
                        "Attempt to close an empty expression",
                        row,
                        col,
                    ));
                } else {
                    return Err(ParseError::boxed(
                        "Too many things in the stack somehow",
                        row,
                        col,
                    ));
                }
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
                let mut tilparend = parse_pkbl(pkbl, gcol, grow)?;
                while matches!(stack.last(), Some(AbstrParam(_))) {
                    if let Some(AbstrParam(c)) = stack.pop() {
                        tilparend = Box::new(Abstr(c, tilparend));
                    } else {
                        unreachable!();
                    }
                }
                if matches!(stack.last(), Some(E(_))) {
                    if let Some(E(before)) = stack.pop() {
                        stack.push(E(Box::new(Appl(before, tilparend))))
                    }
                } else {
                    stack.push(E(tilparend));
                }
                if stack.len() == 1 {
                    if let Some(E(e)) = stack.pop() {
                        return Ok(e);
                    } else {
                        return Err(ParseError::boxed(
                            "Last thing on parser stack is somehow not an expression",
                            row,
                            col,
                        ));
                    }
                } else {
                    return Err(ParseError::boxed(
                        "Too many things in the stack somehow",
                        row,
                        col,
                    ));
                }
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
    if let InExpr = state {
        if stack.len() == 1 {
            if let Some(E(e)) = stack.pop() {
                Ok(e)
            } else {
                Err(ParseError::boxed(
                    "Last thing on parser stack is somehow not an expression",
                    grow,
                    gcol,
                ))
            }
        } else if stack.is_empty() {
            Err(ParseError::boxed("Empty expression", grow, gcol))
        } else {
            Err(ParseError::boxed(
                "Too many things in the stack somehow",
                grow,
                gcol,
            ))
        }
    } else {
        Err(ParseError::boxed(
            "Input ended with abstr parameters",
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
