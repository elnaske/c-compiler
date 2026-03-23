// use std::path::PathBuf;

use crate::lexer::Token;

pub struct CompilerError {
    pub kind: ErrorKind,
    pub filename: String,
    pub line_string: String,
    pub row: usize,
    pub col: usize,
}
impl CompilerError {
    pub fn new(
        kind: ErrorKind,
        filename: String,
        line_string: String,
        row: usize,
        col: usize,
    ) -> Self {
        CompilerError {
            kind,
            filename,
            line_string,
            row,
            col,
        }
    }

    pub fn print(&self) {
        eprintln!(
            "{}:{}:{}: error: {}",
            self.filename,
            self.row,
            self.col,
            self.kind.to_string()
        );
        // TODO: align these two lines better
        eprintln!("{:>5} | {}", self.row, self.line_string);
        eprintln!("{:>5} | {:>width$}", "", "^", width = self.col);
    }
}

pub enum ErrorKind {
    InvalidCharacter,
    InvalidIntSuffix,
    LeftoverTokens,
    Expected { expected: Token, actual: Token },
    // CLIInvalidFlag { flag: String },
    // CLIMissingArg { flag: String, expected: String },
}
impl ErrorKind {
    fn to_string(&self) -> String {
        match self {
            ErrorKind::InvalidCharacter => "Invalid character".to_string(),
            ErrorKind::InvalidIntSuffix => "Invalid suffix on integer constant".to_string(),
            ErrorKind::LeftoverTokens => "Remaining tokens after the end of program".to_string(),
            ErrorKind::Expected { expected, actual } => {
                format!("Expected {:?}, found {:?}", expected, actual)
            } // ErrorKind::CLIInvalidFlag { flag } => format!("Invalid compiler flag `{}`", flag),
              // ErrorKind::CLIMissingArg { flag, expected } => {
              //     format!("Expected {} after `{}`", expected, flag)
              // }
        }
    }
}
