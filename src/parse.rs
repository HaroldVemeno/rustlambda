use std::error::Error;
use std::iter::Peekable;
use std::vec;

use crate::lex::*;

#[derive(Clone, Debug)]
pub enum Expr {
    Variable(u8),
    Name(String),
    Abstr(u8, Box<Expr>),
    Appl(Box<Expr>, Box<Expr>),
}

enum Atom {
    E(Box<Expr>),
    AbstrParam(u8),
}

/*
impl PartialEq for AST {
    fn eq(&self, other: &Self) -> bool {
        use AST::*;
        match (self, other) {
            (Variable(a), Variable(b)) => a == b,
            (Name(a), Name(b)) => a == b,
            (Abstr(a, _), Abstr(_, _)) => todo!(),
            (Appl(_, _), Appl(_, _)) => todo!(),
            _ => false
        }
    }
}
*/

#[derive(Clone)]
pub struct ParseError {
    pub msg: String,
    pub row: u32,
    pub col: u32,
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

#[derive(Copy, Clone, Debug)]
enum ParserState {
    InExpr,
    AbstrInit,
    AbstrParams,
}

pub fn parse(tokps: Vec<TokenPos>) -> Result<Box<Expr>, Box<dyn Error>> {
    parse_iter(&mut tokps.into_iter().peekable())
}

fn parse_iter(pkbl: &mut Peekable<vec::IntoIter<TokenPos>>) -> Result<Box<Expr>, Box<dyn Error>> {
    use Atom::*;
    use Expr::*;
    use ParserState::*;

    let mut state = InExpr;
    let mut stack: Vec<Atom> = vec![];

    let mut grow: u32 = 0;
    let mut gcol: u32 = 0;

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
                let scope = parse_iter(pkbl)?;
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
                            "last thing on parser stack is somehow not an expression",
                            row,
                            col,
                        ));
                    }
                } else {
                    return Err(ParseError::boxed(
                        "too many things in the stack somehow",
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
                let mut tilparend = parse_iter(pkbl)?;
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
                            "last thing on parser stack is somehow not an expression",
                            row,
                            col,
                        ));
                    }
                } else {
                    return Err(ParseError::boxed(
                        "too many things in the stack somehow",
                        row,
                        col,
                    ));
                }
            }
            (s, t) => return Err(ParseError::boxed(
                format!("unexpected token: {:?} while in state {:?}",
                t, s, ), row, col)),
        }
    }
    if let InExpr = state {
        if stack.len() == 1 {
            if let Some(E(e)) = stack.pop() {
                Ok(e)
            } else {
                Err(ParseError::boxed(
                    "last thing on parser stack is somehow not an expression",
                    grow,
                    gcol,
                ))
            }
        } else {
            Err(ParseError::boxed(
                "too many things in the stack somehow",
                grow,
                gcol,
            ))
        }
    } else {
        Err(ParseError::boxed(
            "input ended with abstr parameters",
            grow,
            gcol,
        ))
    }
}
