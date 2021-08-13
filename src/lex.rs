use std::ascii;
use std::error;
use std::error::Error;
use std::fmt;
use std::io::BufReader;
use std::io::Read;

#[derive(Clone, Debug)]
pub enum Token {
    Char(u8),
    Capitalized(String),
    Backslash,
    Dot,
    OpParen,
    ClParen,
}
use Token::*;

#[derive(Clone, Debug)]
pub struct TokenPos {
    pub tok: Token,
    pub row: u32,
    pub col: u32,
}

#[derive(Clone)]
pub struct LexError {
    msg: String,
    row: u32,
    col: u32,
}

pub fn lex(input: impl Read + 'static) -> Result<Vec<TokenPos>, Box<dyn Error>> {
    lex_dyn(Box::new(input))
}

fn lex_dyn(input: Box<dyn Read>) -> Result<Vec<TokenPos>, Box<dyn Error>> {
    let mut p = BufReader::new(input).bytes().peekable();

    let mut vec = Vec::new();

    let mut col: u32 = 1;
    let mut row: u32 = 1;

    macro_rules! err {
        ($($e:expr),+) => {
            Err(Box::new(LexError{row, col, msg: format!($($e),+)}))
        }
    }

    while let Some(Ok(_)) = p.peek() {
        use Token::*;
        let c = p.next().unwrap().unwrap();
        match c {
            b' ' | b'\t' => {}
            b'\n' => {
                row += 1;
                col = 0;
            }
            b'\r' => {
                row += 1;
                col = 0;
                if let Some(Ok(b'\n')) = p.peek() {
                    p.next();
                }
            }
            b'(' => vec.push(TokenPos {
                tok: OpParen,
                col,
                row,
            }),
            b')' => vec.push(TokenPos {
                tok: ClParen,
                col,
                row,
            }),
            b'\\' => vec.push(TokenPos {
                tok: Backslash,
                col,
                row,
            }),
            b'.' => vec.push(TokenPos { tok: Dot, col, row }),
            b'a'..=b'z' => vec.push(TokenPos {
                tok: Char(c),
                col,
                row,
            }),
            b'A'..=b'Z' | b'0'..=b'9' | b'_' => {
                let mut s = (c as char).to_string();
                let scol = col;
                let srow = row;

                while let Some(Ok(c)) = p.peek() {
                    match c {
                        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => s.push((*c).into()),
                        _ => break,
                    }
                    p.next();
                    col += 1;
                }
                vec.push(TokenPos {
                    tok: Capitalized(s),
                    row: srow,
                    col: scol,
                });
            }
            _ => return err!("Bad char '{}'", ascii::escape_default(c)),
        }
        col += 1;
    }
    if let Some(Err(e)) = p.peek() {
        return err!("IO error: {:?}", e);
    }

    Ok(vec)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Char(c) => f.write_str(&ascii::escape_default(*c).to_string()),
            Capitalized(s) => f.write_str(s),
            Backslash => write!(f, "\\"),
            OpParen => write!(f, "("),
            ClParen => write!(f, ")"),
            Dot => write!(f, "."),
        }
    }
}

impl fmt::Display for TokenPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tok.fmt(f)
    }
}

impl error::Error for LexError {}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LexError { row, col, msg } = self;
        write!(f, "LexError({}:{}): {}", row, col, msg)
    }
}

impl fmt::Debug for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LexError { row, col, msg } = self;
        write!(f, "LexError({}:{}): {}", row, col, msg)
    }
}
