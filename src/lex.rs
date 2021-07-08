use std::ascii;
use std::error::Error;
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

#[derive(Clone, Debug)]
pub struct TokenPos {
    pub tok: Token,
    pub row: u32,
    pub col: u32,
}

#[derive(Clone)]
pub struct LexError {
    pub msg: String,
    pub row: u32,
    pub col: u32,
}

pub fn lex<T: Read>(f: T) -> Result<Vec<TokenPos>, Box<dyn Error>> {
    let mut vec = Vec::new();

    let mut col: u32 = 1;
    let mut row: u32 = 1;

    macro_rules! err {
        ($($e:expr),+) => {
            Err(Box::new(LexError{row, col, msg: format!($($e),+)}))
        }
    }

    let mut p = BufReader::new(f).bytes().peekable();
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
            b'A'..=b'Z' | b'0'..=b'9' => {
                let mut s = (c as char).to_string();
                let scol = col;
                let srow = row;

                while let Some(Ok(c)) = p.peek() {
                    match c {
                        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => s.push((*c).into()),
                        _ => break,
                    }
                    p.next();
                    col += 1;
                }
                vec.push(TokenPos {
                    tok: Capitalized(s),
                    col: scol,
                    row: srow,
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
