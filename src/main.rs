#![feature(box_patterns)]

use std::io;
use std::fs::File;
use std::error::Error;
use std::env;

mod eval;
mod lex;
mod parse;
mod print;
mod types;


fn main() -> Result<(), Box<dyn Error>> {
    let vec: Vec<_> = env::args().collect();
    let lexed = if vec.len() == 1 {
        let input = io::stdin();
        eprintln!("Lexing:\n");
        lex::lex(input)?

    } else if vec.len() > 1 {
        let input = File::open(&vec[1])?;
        eprintln!("Lexing:\n");
        lex::lex(input)?
    } else {unreachable!()};


    eprintln!("{:?}", lexed);

    eprintln!("Parsing:\n");
    let parsed = parse::parse(lexed)?;
    eprintln!("{:#}\n", parsed);
    eprintln!("{:?}\n", parsed);
    eprintln!("{}\n", parsed);

    eprintln!("Evaluating:\n");
    let evaluated = eval::reduce_nolog(parsed)?;
    eprintln!("{:#}\n", evaluated);
    eprintln!("{:?}\n", evaluated);
    eprintln!("{}\n", evaluated);

    Ok(())
}
