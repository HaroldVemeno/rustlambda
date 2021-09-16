use crate::expr::Defs;
use crate::{eval, expr, lex, parse};

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::{error, process};

pub fn repl(files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut buf: String = "".into();
    let mut all_defs = expr::Defs::new();
    for f in files {
        let lexed = lex::lex(fs::File::open(f)?)?;
        let (defs, _) = parse::parse(lexed)?;
        for (k, v) in defs {
            all_defs.insert(k, v);
        }
    }
    println!();
    loop {
        let mut cycle = || -> Result<(), Box<dyn Error>> {
            print!("> ");
            io::stdout().flush().expect("flush failed");

            buf.clear();
            io::stdin().read_line(&mut buf)?;

            match command(&buf, &all_defs) {
                Some(Ok(())) => return Ok(()),
                Some(err) => err?,
                None => {}
            }

            let lexed = lex::lex(buf.as_bytes())?;
            let (defs, maybe_expr) = parse::parse(lexed)?;
            for (k, v) in defs {
                all_defs.insert(k, v);
            }
            match maybe_expr {
                Some(e) => {
                    let (evaled, _) = eval::reduce(e, &all_defs);
                    let evaled = evaled?;
                    println!("{}", evaled);
                }
                None => {}
            };
            Ok(())
        };
        match cycle() {
            Ok(()) => {}
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

fn command(line: impl AsRef<str>, defs: &Defs) -> Option<Result<(), Box<dyn Error>>> {
    let trimmed = line.as_ref().trim();
    if trimmed.starts_with(":") {
        let mut rest = &trimmed[1..];
        if let Some((c, _)) = rest.split_once(' ') {
            rest = c
        }

        match rest {
            "quit" | "q" | "exit" => process::exit(0),
            "defs" => {
                for (k, v) in defs {
                    println!("{} = {};", k, v.value)
                }
            }
            "names" => {
                let mut a = "";
                for (k, _) in defs {
                    print!("{}", a);
                    print!("{}", k);
                    a = ", ";
                }
                println!();
            }
            "clear" | "cl" => print!("\x1B[2J\x1B[H"),
            _ => println!("Unknown command: {}", rest),
        }
        Some(Ok(()))
    } else {
        None
    }
}
