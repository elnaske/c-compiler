use crate::errors::{CompilerError, ErrorKind};
use crate::common::{Keyword, UnaryOp, BinaryOp};

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Constant(i32),
    Keyword(Keyword),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Eof,
}


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
                Ok(Token::Eof) => break,
                Ok(token) => tokens.push(token),
                Err(e) => {
                    e.print();
                    panic!()
                }
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
            Some(b'+') => {
                self.advance();
                Ok(Token::BinaryOp(BinaryOp::Add))
            }
            Some(b'-') => {
                self.advance();
                match self.peek() {
                    Some(b'-') => {
                        self.advance();
                        Ok(Token::UnaryOp(UnaryOp::Decrement))
                    }
                    _ => Ok(Token::UnaryOp(UnaryOp::Negation)),
                }
            }
            Some(b'*') => {
                self.advance();
                Ok(Token::BinaryOp(BinaryOp::Mul))
            }
            Some(b'/') => {
                self.advance();
                Ok(Token::BinaryOp(BinaryOp::Div))
            }
            Some(b'%') => {
                self.advance();
                Ok(Token::BinaryOp(BinaryOp::Mod))
            }
            Some(b'~') => {
                self.advance();
                Ok(Token::UnaryOp(UnaryOp::BitwiseComplement))
            }
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
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.peek() {
            self.advance();
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.pos;

        while let Some(b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9') = self.peek() {
            self.advance();
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
                    return Err(CompilerError {
                        kind: ErrorKind::InvalidIntSuffix,
                        filename: "TODO.c".to_string(),
                        line_string: "TODO".to_string(),
                        row: self.row,
                        col: self.col,
                    });
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

    #[test]
    fn return_not_neg_2() {
        let code = b"int main(void) {
        return ~(-2);
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
            Token::UnaryOp(UnaryOp::BitwiseComplement),
            Token::OpenParenthesis,
            Token::UnaryOp(UnaryOp::Negation),
            Token::Constant(2),
            Token::CloseParenthesis,
            Token::Semicolon,
            Token::CloseBrace,
        ];

        assert_eq!(ref_tokens, tokens);
        assert_eq!(lexer.pos, 0);
    }

    #[test]
    fn negation_vs_decrement() {
        let code_neg = b"-2";
        let code_dec = b"--2";

        let neg_tokens = Lexer::new(code_neg).get_tokens();
        let dec_tokens = Lexer::new(code_dec).get_tokens();

        assert_eq!(
            neg_tokens,
            vec![Token::UnaryOp(UnaryOp::Negation), Token::Constant(2)]
        );
        assert_eq!(
            dec_tokens,
            vec![Token::UnaryOp(UnaryOp::Decrement), Token::Constant(2)]
        );
    }
}
