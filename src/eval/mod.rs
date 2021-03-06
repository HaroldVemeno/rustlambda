use std::{error, fmt};

pub mod eval;
pub mod util;

pub use eval::reduce;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct EvalError {
    msg: String,
}

impl EvalError {
    fn boxed(msg: impl Into<String>) -> Box<EvalError> {
        Box::new(EvalError { msg: msg.into() })
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {}", self.msg)
    }
}

impl fmt::Debug for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {}", self.msg)
    }
}

impl error::Error for EvalError {}
