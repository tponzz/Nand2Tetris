use std::{fmt, fs::File, io::BufWriter, io::Write};

use crate::JAError;

#[derive(Debug)]
struct VmWriter {
    writer: BufWriter<File>,
}

#[derive(Debug)]
enum SegmentKind {
    Constant,
    Argument,
    Local,
    Static,
    This,
    That,
    Pointer,
    Temp,
}

impl fmt::Display for SegmentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentKind::Constant => write!(f, "constant"),
            SegmentKind::Argument => write!(f, "argument"),
            SegmentKind::Local => write!(f, "local"),
            SegmentKind::Static => write!(f, "static"),
            SegmentKind::This => write!(f, "this"),
            SegmentKind::That => write!(f, "that"),
            SegmentKind::Pointer => write!(f, "pointer"),
            SegmentKind::Temp => write!(f, "temp"),
        }
    }
}

#[derive(Debug)]
enum CommandKind {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

impl fmt::Display for CommandKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandKind::Add => write!(f, "add"),
            CommandKind::Sub => write!(f, "sub"),
            CommandKind::Neg => write!(f, "neg"),
            CommandKind::Eq => write!(f, "eq"),
            CommandKind::Gt => write!(f, "gt"),
            CommandKind::Lt => write!(f, "lt"),
            CommandKind::And => write!(f, "and"),
            CommandKind::Or => write!(f, "or"),
            CommandKind::Not => write!(f, "not"),
        }
    }
}

impl VmWriter {
    fn new(source: &str) -> Self {
        let file = File::open(source).expect("Failed to open output file");
        let writer = BufWriter::new(file);
        Self { writer }
    }

    pub fn write_push(&mut self, segment: &SegmentKind, index: u32) -> Result<(), JAError> {
        writeln!(self.writer, "push {} {}", segment, index)?;
        Ok(())
    }

    pub fn write_pop(&mut self, segment: &SegmentKind, index: u32) -> Result<(), JAError> {
        writeln!(self.writer, "pop {} {}", segment, index)?;
        Ok(())
    }

    pub fn write_arithmetic(&mut self, command: &SegmentKind) -> Result<(), JAError> {
        writeln!(self.writer, "{}", command)?;
        Ok(())
    }

    pub fn write_label(&mut self, label: &str) -> Result<(), JAError> {
        writeln!(self.writer, "label {}", label)?;
        Ok(())
    }

    pub fn write_if(&mut self, label: &str) -> Result<(), JAError> {
        writeln!(self.writer, "if-goto {}", label)?;
        Ok(())
    }

    pub fn write_goto(&mut self, label: &str) -> Result<(), JAError> {
        writeln!(self.writer, "goto {}", label)?;
        Ok(())
    }

    pub fn write_call(&mut self, func_name: &str, n_vars: &str) -> Result<(), JAError> {
        writeln!(self.writer, "call {} {}", func_name, n_vars)?;
        Ok(())
    }

    pub fn write_function(&mut self, func_name: &str, n_vars: &str) -> Result<(), JAError> {
        writeln!(self.writer, "function {} {}", func_name, n_vars)?;
        Ok(())
    }

    pub fn write_return(&mut self) -> Result<(), JAError> {
        writeln!(self.writer, "return")?;
        Ok(())
    }
}
