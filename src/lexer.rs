use crate::common::{Keyword, Operator};
use crate::errors::{CompilerError, ErrorKind};

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Constant(i32),
    Keyword(Keyword),
    // UnaryOp(UnaryOp),
    // BinaryOp(BinaryOp),
    Operator(Operator),
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Eof,
}

pub struct Lexer<'a> {
    input: &'a [u8],
    filename: String,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8], filename: String) -> Self {
        Lexer {
            input,
            filename,
            pos: 0,
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
                Ok(Token::Operator(Operator::Plus))
            }
            Some(b'-') => {
                self.advance();
                match self.peek() {
                    Some(b'-') => {
                        self.advance();
                        Ok(Token::Operator(Operator::Decrement))
                    }
                    _ => Ok(Token::Operator(Operator::Minus)),
                }
            }
            Some(b'*') => {
                self.advance();
                Ok(Token::Operator(Operator::Mul))
            }
            Some(b'/') => {
                self.advance();
                Ok(Token::Operator(Operator::Div))
            }
            Some(b'%') => {
                self.advance();
                Ok(Token::Operator(Operator::Mod))
            }
            Some(b'~') => {
                self.advance();
                Ok(Token::Operator(Operator::BitwiseNot))
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
            other => Err(CompilerError::new(ErrorKind::InvalidCharacter(other.unwrap()), self.filename.clone(), self.pos)),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).cloned()
    }

    fn advance(&mut self) {
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
                    return Err(CompilerError::new(ErrorKind::InvalidIntSuffix, self.filename.clone(), self.pos));
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

        let mut lexer = Lexer::new(code, "foo.c".to_string());
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

        let mut lexer = Lexer::new(code, "foo.c".to_string());
        let tokens = lexer.get_tokens();

        let ref_tokens = vec![
            Token::Keyword(Keyword::Int),
            Token::Identifier("main".to_string()),
            Token::OpenParenthesis,
            Token::Keyword(Keyword::Void),
            Token::CloseParenthesis,
            Token::OpenBrace,
            Token::Keyword(Keyword::Return),
            Token::Operator(Operator::BitwiseNot),
            Token::OpenParenthesis,
            Token::Operator(Operator::Minus),
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

        let neg_tokens = Lexer::new(code_neg, "foo.c".to_string()).get_tokens();
        let dec_tokens = Lexer::new(code_dec, "foo.c".to_string()).get_tokens();

        assert_eq!(
            neg_tokens,
            vec![Token::Operator(Operator::Minus), Token::Constant(2)]
        );
        assert_eq!(
            dec_tokens,
            vec![Token::Operator(Operator::Decrement), Token::Constant(2)]
        );
    }
}
