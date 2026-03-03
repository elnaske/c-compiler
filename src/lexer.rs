#[derive(Debug, PartialEq)]
enum Token {
    Identifier(String),
    Constant(i32),
    Keyword(Keyword),
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
}

#[derive(Debug, PartialEq)]
enum Keyword {
    Int,
    Void,
    Return,
}

impl Keyword {
    // fn from_str(s: &str) -> Option<Self> {
    //     match s {
    //         "int" => Some(Keyword::Int),
    //         "void" => Some(Keyword::Void),
    //         "return" => Some(Keyword::Return),
    //         _ => None,
    //     }
    // }
    fn from_u8(s: &[u8]) -> Option<Self> {
        match s {
            b"int" => Some(Keyword::Int),
            b"void" => Some(Keyword::Void),
            b"return" => Some(Keyword::Return),
            _ => None,
        }
    }
}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Self {
        Lexer { input, pos: 0 }
    }

    fn get_tokens(&mut self) -> Vec<Token> {
        // parse tokens (self.next_token) until EOF
        let mut tokens = Vec::<Token>::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        match self.peek() {
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => Some(self.lex_identifier()),
            Some(b'0'..=b'9') => Some(self.lex_constant()),
            Some(b'(') => {
                self.advance();
                Some(Token::OpenParenthesis)
            }
            Some(b')') => {
                self.advance();
                Some(Token::CloseParenthesis)
            }
            Some(b'{') => {
                self.advance();
                Some(Token::OpenBrace)
            }
            Some(b'}') => {
                self.advance();
                Some(Token::CloseBrace)
            }
            Some(b';') => {
                self.advance();
                Some(Token::Semicolon)
            }
            None => None,
            _ => panic!("illegal character"),
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos >= self.input.len() {
            return None;
        }
        Some(self.input[self.pos])
    }

    fn advance(&mut self) {
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

    fn lex_constant(&mut self) -> Token {
        let start = self.pos;

        loop {
            match self.peek() {
                Some(b'0'..=b'9') => self.advance(),
                _ => break,
            }
        }
        Token::Constant(
            str::from_utf8(&self.input[start..self.pos])
                .expect("Invalid UTF-8 sequence")
                .parse()
                .expect("Not a valid number"),
        )
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

        let tokens = Lexer::new(code).get_tokens();

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
    }
}
