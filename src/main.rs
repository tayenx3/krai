mod lexer;
mod operator;
mod span;
mod diagnostic;

use clap::Parser;
use tinycolor::Colorize;
use std::{env, fs};

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
    println!("{tokens:#?}");
}
