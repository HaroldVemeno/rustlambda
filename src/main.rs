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
    if vec.len() == 1 {
        let input = io::stdin();
        eprintln!("Lexing:\n");
        let lres = lex::lex(input)?;
        eprintln!("{:?}", lres);
        eprintln!("Parsing:\n");
        let pres = parse::parse(lres)?;
        eprintln!("{:#}\n", pres);
        eprintln!("{:?}\n", pres);
        eprintln!("{}\n", pres);
    } else if vec.len() > 1 {
        let file = File::open(&vec[1])?;
        eprintln!("Lexing:\n");
        let lres = lex::lex(file)?;
        eprintln!("{:?}", lres);
        eprintln!("Parsing:\n");
        let pres = parse::parse(lres)?;
        eprintln!("{:#}\n", pres);
        eprintln!("{:?}\n", pres);
        eprintln!("{}\n", pres);
    }

    Ok(())
}
