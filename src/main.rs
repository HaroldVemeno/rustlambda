#[allow(unused_imports)]
use rustlambda::{eval, expr, lex, parse, repl};

use std::error::Error;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Lambda calculus evaluator and more")]
enum Opt {
    #[structopt(alias = "r")]
    Repl {
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>,
    },
    #[structopt(alias = "e")]
    Eval {
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>,
    },
    #[structopt(alias = "h")]
    Help,
}

fn main() -> Result<(), Box<dyn Error>> {
    use Opt::*;
    let opt = Opt::from_args();
    match opt {
        Help => {
            Opt::clap().print_long_help()?;
            println!();
        }
        Eval { mut files } => {
            let filecount = files.len();
            if filecount == 0 {
                files.push("-".into());
            }

            for (i, file) in files.into_iter().enumerate() {
                let lexed = if file == Path::new("-") {
                    eprintln!("Processing from stdin ({}/{}):", i + 1, filecount);
                    eprintln!("Lexing...");
                    lex::lex(io::stdin())
                } else {
                    eprintln!(
                        "Processing {} ({}/{}):",
                        file.to_string_lossy(),
                        i + 1,
                        filecount
                    );
                    eprintln!("Lexing...");
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
        Repl { files } => repl::repl(files)?,
    };
    Ok(())
}
