#![feature(box_patterns)]

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;

mod eval;
mod lex;
mod parse;
mod expr;

fn main() -> Result<(), Box<dyn Error>> {
    let vec: Vec<_> = env::args().collect();
    let lexed = match vec.len() {
        1 => {
            let input = io::stdin();
            eprintln!("Lexing:\n");
            lex::lex(input)?
        }
        _ => {
            let input = File::open(&vec[1])?;
            eprintln!("Lexing:\n");
            lex::lex(input)?
        }
    };
    //eprintln!("{:?}", lexed);

    eprintln!("Parsing\n");
    let parsed = parse::parse(lexed)?;
    // eprintln!("{:#}\n", parsed);
    // eprintln!("{:?}\n", parsed);
    // eprintln!("{}\n", parsed);

    eprintln!("Evaluating\n");
    let evaluated = eval::reduce_nolog(parsed)?;
    // eprintln!("{:#}\n", evaluated);
    // eprintln!("{:?}\n", evaluated);
    eprintln!("{}\n", evaluated);

    Ok(())
}
