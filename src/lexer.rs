use std::path::PathBuf;

use crate::errors::{CompilerError, ErrorKind};

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Constant(i32),
    Keyword(Keyword),
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Eof,
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Int,
    Void,
    Return,
}

impl Keyword {
    fn from_u8(s: &[u8]) -> Option<Self> {
        match s {
            b"int" => Some(Keyword::Int),
            b"void" => Some(Keyword::Void),
            b"return" => Some(Keyword::Return),
            _ => None,
        }
    }
}

// TODO:
// - what happens to Lexer when done? reset automatically or manually?
// - better error handling
pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    row: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Lexer {
            input,
            pos: 0,
            row: 1,
            col: 1,
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::<Token>::new();
        loop {
            match self.next_token() {
                Ok(token) if token == Token::Eof => break,
                Ok(token) => tokens.push(token),
                Err(e) => {e.print(); panic!()},
            }
        }
        self.reset();
        tokens
    }

    fn next_token(&mut self) -> Result<Token, CompilerError> {
        self.skip_whitespace();

        match self.peek() {
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => Ok(self.lex_identifier()),
            Some(b'0'..=b'9') => self.lex_constant(),
            Some(b'(') => {
                self.advance();
                Ok(Token::OpenParenthesis)
            }
            Some(b')') => {
                self.advance();
                Ok(Token::CloseParenthesis)
            }
            Some(b'{') => {
                self.advance();
                Ok(Token::OpenBrace)
            }
            Some(b'}') => {
                self.advance();
                Ok(Token::CloseBrace)
            }
            Some(b';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            None => Ok(Token::Eof),
            _ => Err(CompilerError {
                kind: ErrorKind::InvalidCharacter,
                filename: "TODO.c".to_string(),
                line_string: "TODO".to_string(),
                row: self.row,
                col: self.col,
            }),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).cloned()
    }

    fn advance(&mut self) {
        if self.peek() == Some(b'\n') {
            self.row += 1; // this doesn't work right; idk why
            self.col = 1;
        } else {
            self.col += 1
        }
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\n' | b'\r') => self.advance(),
                _ => break,
            }
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.pos;

        loop {
            match self.peek() {
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9') => self.advance(),
                _ => break,
            }
        }
        if let Some(keyword) = Keyword::from_u8(&self.input[start..self.pos]) {
            Token::Keyword(keyword)
        } else {
            Token::Identifier(
                str::from_utf8(&self.input[start..self.pos])
                    .expect("Invalid UTF-8 sequence")
                    .to_string(),
            )
        }
    }

    fn lex_constant(&mut self) -> Result<Token, CompilerError> {
        let start = self.pos;

        loop {
            match self.peek() {
                Some(b'0'..=b'9') => self.advance(),
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
                    // panic!("invalid suffix on integer constant")
                    return Err(CompilerError{
                        kind: ErrorKind::InvalidIntSuffix,
                        filename: "TODO.c".to_string(),
                        line_string: "TODO".to_string(),
                        row: self.row,
                        col: self.col,
                    })
                } // put this here to satisfy a test case, might change how this is handled in the future
                _ => break,
            }
        }
        Ok(Token::Constant(
            str::from_utf8(&self.input[start..self.pos])
                .expect("Invalid UTF-8 sequence")
                .parse()
                .expect("Not a valid number"),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn return_2() {
        let code = b"int main(void) {
        return 2;
        }";

        let mut lexer = Lexer::new(code);
        let tokens = lexer.get_tokens();

        let ref_tokens = vec![
            Token::Keyword(Keyword::Int),
            Token::Identifier("main".to_string()),
            Token::OpenParenthesis,
            Token::Keyword(Keyword::Void),
            Token::CloseParenthesis,
            Token::OpenBrace,
            Token::Keyword(Keyword::Return),
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace,
        ];

        assert_eq!(ref_tokens, tokens);
        assert_eq!(lexer.pos, 0);
    }
}
