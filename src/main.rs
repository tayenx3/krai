mod lexer;
mod parser;
mod sema;
mod operator;
mod span;
mod diagnostic;

use clap::Parser;
use tinycolor::Colorize;
use std::{env, fs, path::PathBuf, time::Instant};

#[derive(Parser)]
#[command(
    bin_name = env!("CARGO_BIN_NAME"),
    about = "The Krai Language Compiler",
    version,
)]
struct Cli {
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    no_color: bool,
}

fn main() {
    let cli = Cli::parse();
    let error_prefix = if !cli.no_color {
        "error".red().bold().to_string()
    } else {
        "error".to_string()
    };

    let start_time = Instant::now();

    let source = match fs::read_to_string(&cli.input) {
        Ok(src) => src,
        Err(error) => {
            eprintln!("{error_prefix}: {error}");
            eprintln!("{error_prefix}: compilation aborted due to {} previous errors", if !cli.no_color {
                "1".red().bold().to_string()
            } else {
                "1".to_string()
            });
            return;
        }
    };

    let mut rodeo = lasso::Rodeo::new();

    let mut line_starts = vec![0];
    for (pos, ch) in source.char_indices() {
        if ch == '\n' {
            line_starts.push(pos + 1);
        }
    }

    let tokens = match lexer::tokenize(&cli.input, &source, cli.no_color, &mut rodeo) {
        Ok(toks) => toks,
        Err(error) => {
            eprintln!("{}", error.format(&line_starts, &source.lines().collect::<Vec<_>>()));
            eprintln!("{error_prefix}: compilation aborted due to {} previous errors", if !cli.no_color {
                "1".red().bold().to_string()
            } else {
                "1".to_string()
            });
            return;
        },
    };

    let ast = match parser::Parser::new(&rodeo, &cli.input, &tokens, cli.no_color).parse() {
        Ok(ast) => ast,
        Err(error) => {
            eprintln!("{}", error.format(&line_starts, &source.lines().collect::<Vec<_>>()));
            eprintln!("{error_prefix}: compilation aborted due to {} previous errors", if !cli.no_color {
                "1".red().bold().to_string()
            } else {
                "1".to_string()
            });
            return;
        },
    };

    let mut sem_checker = sema::SemChecker::new(&mut rodeo, &cli.input, cli.no_color);
    let type_map = match sem_checker.check(&ast) {
        Ok(()) => sem_checker.type_map,
        Err(errors) => {
            let lines = source.lines().collect::<Vec<_>>();
            for error in &errors {
                eprintln!("{}", error.format(&line_starts, &lines));
            }
            eprintln!("{error_prefix}: compilation aborted due to {} previous errors", if !cli.no_color {
                errors.len().to_string().red().bold().to_string()
            } else {
                errors.len().to_string()
            });
            return;
        },
    };
    let _ = type_map;

    let _output_path = cli.output.as_ref()
        .map(|n| PathBuf::from(n))
        .unwrap_or(PathBuf::from(&cli.input).with_extension("exe"));

    println!("{} compilation in {:.3}s",
        if !cli.no_color {
            "finished".green().bold().to_string()
        } else {
            "finished".to_string()
        },
        start_time.elapsed().as_secs_f32()
    );
}
