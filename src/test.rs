pub use crate::eval;
pub use crate::expr::{self, Expr};
pub use crate::lex::{self, lex};
pub use crate::parse::{self, parse};

pub fn process(s: &'static str) -> Box<Expr> {
    lex(s.as_bytes()).and_then(parse).unwrap().1.unwrap()
}
