mod cli;
mod tokenizer;

use clap::Parser;
use tokenizer::Tokenizer;

use crate::tokenizer::TokenType;
#[derive(Debug)]
pub enum JAError {
    Io,
}

pub fn run() -> Result<(), JAError> {
    let cli = cli::Args::parse();
    println!("{}", cli.source);

    let mut tokenizer = Tokenizer::new(&cli.source);
    while tokenizer.has_more_tokens() {
        if tokenizer.advance().is_err() {
            return Err(JAError::Io);
        };

        if let Some(t) = tokenizer.token_type() {
            match t {
                TokenType::Keyword => {
                    if let Some(key) = tokenizer.keyword() {
                        println!("KEY: {:?}", key);
                    }
                }
                TokenType::Symbol => {
                    if let Some(sym) = tokenizer.symbol() {
                        println!("SYM: {}", sym);
                    }
                }
                TokenType::Identifier => {
                    if let Some(id) = tokenizer.identifier() {
                        println!("IDT: {}", id);
                    }
                }
                TokenType::IntConst => {
                    if let Some(val) = tokenizer.int_val() {
                        println!("INT: {}", val);
                    }
                }
                TokenType::StringConst => {
                    if let Some(val) = tokenizer.string_val() {
                        println!("STR: {}", val);
                    }
                }
            }
        };
    }

    Ok(())
}
