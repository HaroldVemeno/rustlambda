#[allow(unused_imports)]
use rustlambda::{eval, expr, lex, parse};

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Lambda calculus evaluator and more")]
enum Opt {
    // #[structopt(alias = "r")]
    // Repl {
    //     #[structopt(parse(from_os_str))]
    //     files: Vec<PathBuf>,
    // },
    #[structopt(alias = "e")]
    Eval {
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>,
    },
    // #[structopt(alias = "h")]
    // Help {},
}

fn main() -> Result<(), Box<dyn Error>> {
    use Opt::*;
    let opt = Opt::from_args();
    dbg!(&opt);
    match opt {
        Eval { mut files } => {
            if files.is_empty() {
                files.push("-".into());
            }
            for file in files {
                eprintln!("Lexing...");
                let lexed = if file == Path::new("-") {
                    lex::lex(io::stdin())
                } else {
                    lex::lex(File::open(file)?)
                }?;
                //eprintln!("{:?}", lexed);

                eprintln!("Parsing...");
                let (defs, m_expr) = parse::parse(lexed)?;
                let expr = m_expr.ok_or("No main body to evaluate")?;
                //eprintln!("{:#}\n", expr);
                //eprintln!("{:?}\n", expr);
                for (k, v) in defs.iter() {
                    eprintln!("{} = {};", k, v.value);
                }
                eprintln!("{}\n", expr);

                eprintln!("Evaluating...");
                let (eval_res, stats) = eval::reduce(expr, &defs);
                let evaluated = eval_res?;

                println!("{}\n", evaluated);
                if let Some(num) = evaluated.try_unchurch_num() {
                    eprintln!("Church num!: {}", num)
                }
                eprintln!("{}\n", stats);
                // eprintln!("{:#}\n", evaluated);
                // eprintln!("{:?}\n", evaluated);
            }
        }
    };
    Ok(())
}

fn oldmain() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    eprintln!("Lexing...");
    let lexed = match args.next() {
        None => lex::lex(io::stdin()),
        Some(s) => lex::lex(File::open(&s)?),
    }?;
    //eprintln!("{:?}", lexed);

    eprintln!("Parsing...");
    let parsed = parse::parse(lexed)?;
    let expr = parsed.1.ok_or("No main body to evaluate")?;
    let defs = parsed.0;
    //eprintln!("{:#}\n", expr);
    //eprintln!("{:?}\n", expr);
    eprintln!("{}\n", expr);

    eprintln!("Evaluating\n");
    let (eval_res, stats) = eval::reduce(expr, &defs);
    let evaluated = eval_res?;

    println!("{}\n", evaluated);
    if let Some(num) = evaluated.try_unchurch_num() {
        eprintln!("Church num!: {}", num)
    }
    eprintln!("{}\n", stats);
    // eprintln!("{:#}\n", evaluated);
    // eprintln!("{:?}\n", evaluated);

    Ok(())
}
