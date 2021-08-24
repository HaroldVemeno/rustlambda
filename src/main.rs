#[allow(unused_imports)]
use rustlambda::{eval, expr, lex, parse};

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    eprintln!("Lexing:\n");
    let lexed = match args.next() {
        None => lex::lex(io::stdin()),
        Some(s) => lex::lex(File::open(&s)?),
    }?;
    //eprintln!("{:?}", lexed);

    eprintln!("Parsing\n");
    let parsed = parse::parse(lexed)?.1.ok_or("No main body to evaluate")?;
    //eprintln!("{:#}\n", parsed);
    //eprintln!("{:?}\n", parsed);
    eprintln!("{}\n", parsed);

    eprintln!("Evaluating\n");
    let evaluated = eval::reduce(parsed, true)?;
    println!("{}\n", evaluated);
    if let Some(num) = evaluated.try_unchurch_num() {
        eprintln!("Church num!: {}", num)
    }
    // eprintln!("{:#}\n", evaluated);
    // eprintln!("{:?}\n", evaluated);

    Ok(())
}
