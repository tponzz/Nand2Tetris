mod cli;
mod tokenizer;

use clap::Parser;
use tokenizer::Tokenizer;
#[derive(Debug)]
pub enum JAError {
    Io,
}

pub fn run() -> Result<(), JAError> {
    let cli = cli::Args::parse();
    println!("{}", cli.source);

    let mut tokenizer = Tokenizer::new(&cli.source);
    while tokenizer.has_more_tokens() {
        tokenizer.advance();

        //TODO
        println!(
            "keyword: {:?}, type: {:?}",
            tokenizer.keyword(),
            tokenizer.token_type()
        );
    }
    Err(JAError::Io)
}
