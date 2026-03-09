use crate::lexer::{Keyword, Token};
use std::fmt::{self, Formatter};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub function: Function,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Program({})", self.function)
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Statement,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Function(name='{}', body={})", self.name, self.body,)
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Return(exp) => {
                write!(f, "Return({})", exp)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Constant(i32),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Constant(i) => write!(f, "Constant({})", i),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens,
            pos: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let program = Program {
            function: self.parse_function()?,
        };

        if self.pos < self.tokens.len() {
            Err("leftover tokens after program".to_string())
        }
        else {
            Ok(program)
        }
    }

    // transfer ownership instead of returning reference?
    fn take_token(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let actual = self.take_token();
        if actual == Some(&expected) {
            Ok(())
        } else {
            Err(format!(
                "Syntax error: expected '{:?}', found '{:?}'",
                expected, actual
            ))
        }
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Keyword(Keyword::Int))?;

        let name = self.parse_identifier()?;

        self.expect(Token::OpenParenthesis)?;
        self.expect(Token::Keyword(Keyword::Void))?;
        self.expect(Token::CloseParenthesis)?;
        self.expect(Token::OpenBrace)?;

        let body = self.parse_statement()?;

        self.expect(Token::CloseBrace)?;

        Ok(Function { name, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::Keyword(Keyword::Return))?;

        let expression = self.parse_expression()?;

        self.expect(Token::Semicolon)?;

        Ok(Statement::Return(expression))
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.take_token() {
            Some(Token::Identifier(s)) => Ok(s.clone()),
            other => Err(format!("Expected identifier, found {:?}", other)),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        match self.take_token() {
            Some(Token::Constant(i)) => Ok(Expression::Constant(*i)),
            other => Err(format!("Expected expression, found {:?}", other)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn return_2() {
        let code = b"int main(void) {
        return 2;
        }";

        let mut lexer = Lexer::new(code);
        let tokens = lexer.get_tokens();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let ref_program = Program {
            function: Function {
                name: "main".to_string(),
                body: Statement::Return(Expression::Constant(2)),
            },
        };

        assert_eq!(program, ref_program)
    }
}
