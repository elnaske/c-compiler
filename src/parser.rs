use std::fmt::{self, Formatter};
use crate::lexer::{Keyword, Token};

pub struct Program {
    function: Function,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Program(
                {}
            )",
            self.function)
    }
}

struct Function {
    identifier: String,
    body: Statement,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function(
                name='{}',
                body={}
            )",
            self.identifier,
            self.body,
        )
    }
}

enum Statement {
    Return(Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Return(exp) => write!(f, "Return(\n\t{}\n)", exp)
        }
    }
}

enum Expression {
    Constant(i32),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Constant(i) => write!(f, "Constant({})", i)
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn parse_program(&mut self) -> Option<Program>{
        while let Some(token) = self.take_token() {
            
        }
        unimplemented!() 
    }
    
    fn take_token(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if let Some(actual) = self.take_token() {
            if *actual != expected {
                Err(format!("Syntax error: expected '{:?}', found '{:?}'", expected, *actual))
            } else {
                Ok(())
            }
        } else {
            Err("End of tokens".to_string())
        }
    }

    fn parse_function(&mut self) -> Option<Function> {
        self.expect(Token::Keyword(Keyword::Int)).ok()?;
        let identifier = self.parse_identifier().expect("Invalid identifier");
        self.expect(Token::OpenParenthesis).ok()?;
        self.expect(Token::Keyword(Keyword::Void)).ok()?;
        self.expect(Token::CloseParenthesis).ok()?;
        self.expect(Token::OpenBrace).ok()?;
        let body = self.parse_statement().expect("Invalid statement");
        self.expect(Token::CloseBrace).ok()?;
        Some(Function { identifier, body })
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        self.expect(Token::Keyword(Keyword::Return)).ok()?;
        let return_value = self.parse_expression().expect("Invalid expression");
        self.expect(Token::Semicolon).ok()?;
        Some(Statement::Return(return_value))
    }

    fn parse_identifier(&mut self) -> Option<String> {
        match self.take_token() {
            Some(Token::Identifier(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        match self.take_token() {
            Some(Token::Constant(i)) => Some(Expression::Constant(*i)),
            _ => None,
        }
    }
}
