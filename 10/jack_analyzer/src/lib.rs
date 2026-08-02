mod cli;
mod engine;
mod tokenizer;

use std::process::exit;

use crate::engine::CompilationEngine;
use clap::Parser;

#[derive(Debug)]
pub enum JAError {
    Io,
    Compile(CompileErrKind),
}

#[derive(Debug, Clone)]
pub enum CompileErrKind {
    Class,
    Subroutine,
    ParameterList,
    SubroutineBody,
    VarDec,
    Let,
    If,
    While,
    Do,
    Return,
    Term,
}

pub fn run() -> Result<(), JAError> {
    let cli = cli::Args::parse();
    println!("{}", cli.source);

    let sink = "Out.xml";

    match CompilationEngine::new(&cli.source, sink) {
        Ok(mut engine) => engine.compile_class(),
        Err(e) => {
            eprintln!("Failed to open files: {:?}", e);
            exit(1)
        }
    }
}
