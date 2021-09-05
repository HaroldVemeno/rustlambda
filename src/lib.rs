#![feature(box_patterns)]

pub mod eval;
#[macro_use]
pub mod expr;
pub mod lex;
pub mod parse;

#[cfg(test)]
pub mod test;
