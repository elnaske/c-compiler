use std::fmt;

use crate::lexer::Token;

fn find_line_and_col(_filename: &str, _pos: usize) -> (usize, usize) {
    unimplemented!()
}

fn get_full_line(_line_num: usize) -> String {
    unimplemented!()
}

pub struct CompilerError {
    pub kind: ErrorKind,
    pub filename: String,
    pub line_string: String,
    pub line_num: usize,
    pub col: usize,
}
impl CompilerError {
    pub fn new(kind: ErrorKind, filename: String, pos: usize) -> Self {
        let (line_num, col) = find_line_and_col(&filename, pos);
        let line_string = get_full_line(line_num);

        CompilerError {
            kind,
            filename,
            line_string,
            line_num,
            col,
        }
    }

    pub fn print(&self) {
        eprintln!(
            "{}:{}:{}: error: {}",
            self.filename, self.line_num, self.col, self.kind,
        );
        eprintln!("{:>5} | {}", self.line_num, self.line_string);
        eprintln!("{:>5} | {:>width$}", "", "^", width = self.col);
    }
}

pub enum ErrorKind {
    InvalidCharacter(u8),
    InvalidIntSuffix,
    LeftoverTokens,
    Expected { expected: Token, actual: Token },
    // CLIInvalidFlag { flag: String },
    // CLIMissingArg { flag: String, expected: String },
}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "Invalid character: `{}`", c),
            Self::InvalidIntSuffix => write!(f, "Invalid suffix on integer constant"),
            Self::LeftoverTokens => write!(f, "Remaining tokens after the end of program"),
            Self::Expected { expected, actual } => {
                write!(f, "Expected {:?}, found {:?}", expected, actual)
            } // ErrorKind::CLIInvalidFlag { flag } => format!("Invalid compiler flag `{}`", flag),
              // ErrorKind::CLIMissingArg { flag, expected } => {
              //     format!("Expected {} after `{}`", expected, flag)
              // }
        }
    }
}
