mod engine;
mod symbol_table;
mod tokenizer;
mod vm_writer;

use std::{
    io::{self},
    process::exit,
};

use crate::engine::CompilationEngine;
use clap::Parser;

#[derive(clap::Parser)]
#[command(version, about)]
pub struct Args {
    pub source: String,
}

#[derive(Debug)]
pub enum JAError {
    Io(String),
    Compile(CompileErrKind),
}

impl From<io::Error> for JAError {
    fn from(e: io::Error) -> Self {
        JAError::Io(e.to_string())
    }
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
    let cli = Args::parse();
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
