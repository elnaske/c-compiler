use crate::common::{Keyword, UnaryOp, BinaryOp};
use crate::lexer::Token;
use std::fmt::{self, Formatter};

// TODO: factor out ASTs into separate files

#[derive(Debug, PartialEq)]
pub struct CProgram {
    pub function: CFunction,
}

impl fmt::Display for CProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Program({})", self.function)
    }
}

#[derive(Debug, PartialEq)]
pub struct CFunction {
    pub name: String,
    pub body: CStatement,
}

impl fmt::Display for CFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Function(name='{}', body={})", self.name, self.body,)
    }
}

#[derive(Debug, PartialEq)]
pub enum CStatement {
    Return(CExpression),
}

impl fmt::Display for CStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(exp) => {
                write!(f, "Return({})", exp)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CExpression {
    Constant(i32),
    Unary(UnaryOp, Box<CExpression>),
    Binary(BinaryOp, Box<CExpression>, Box<CExpression>),
}

impl fmt::Display for CExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(i) => write!(f, "Constant({})", i),
            Self::Unary(op, exp) => write!(f, "UnaryOp({}, {})", op, *exp),
            Self::Binary(op, exp1, exp2) => write!(f, "BinaryOp({}, {}, {})", op, *exp1, *exp2),
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
            tokens,
            pos: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<CProgram, String> {
        let program = CProgram {
            function: self.parse_function()?,
        };

        if self.pos < self.tokens.len() {
            Err("leftover tokens after program".to_string())
        } else {
            Ok(program)
        }
    }

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

    fn parse_function(&mut self) -> Result<CFunction, String> {
        self.expect(Token::Keyword(Keyword::Int))?;

        let name = self.parse_identifier()?;

        self.expect(Token::OpenParenthesis)?;
        self.expect(Token::Keyword(Keyword::Void))?;
        self.expect(Token::CloseParenthesis)?;
        self.expect(Token::OpenBrace)?;

        let body = self.parse_statement()?;

        self.expect(Token::CloseBrace)?;

        Ok(CFunction { name, body })
    }

    fn parse_statement(&mut self) -> Result<CStatement, String> {
        self.expect(Token::Keyword(Keyword::Return))?;

        let expression = self.parse_expression()?;

        self.expect(Token::Semicolon)?;

        Ok(CStatement::Return(expression))
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.take_token() {
            Some(Token::Identifier(s)) => Ok(s.clone()),
            other => Err(format!("Expected identifier, found {:?}", other)),
        }
    }

    fn parse_expression(&mut self) -> Result<CExpression, String> {
        match self.take_token() {
            Some(Token::Constant(i)) => Ok(CExpression::Constant(*i)),
            Some(Token::UnaryOp(UnaryOp::Decrement)) => unimplemented!(),
            Some(Token::UnaryOp(unop)) => Ok(CExpression::Unary(
                unop.clone(),
                Box::new(self.parse_expression()?),
            )),
            Some(Token::OpenParenthesis) => {
                let inner_exp = self.parse_expression()?;
                self.expect(Token::CloseParenthesis)?;
                Ok(inner_exp)
            }
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

        let ref_program = CProgram {
            function: CFunction {
                name: "main".to_string(),
                body: CStatement::Return(CExpression::Constant(2)),
            },
        };

        assert_eq!(program, ref_program)
    }

    #[test]
    fn return_neg_2() {
        let code = b"int main(void) {
        return -2;
        }";

        let mut lexer = Lexer::new(code);
        let tokens = lexer.get_tokens();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let ref_program = CProgram {
            function: CFunction {
                name: "main".to_string(),
                body: CStatement::Return(CExpression::Unary(
                    UnaryOp::Negation,
                    Box::new(CExpression::Constant(2)),
                )),
            },
        };

        assert_eq!(program, ref_program)
    }

    #[test]
    fn return_not_2() {
        let code = b"int main(void) {
        return ~2;
        }";

        let mut lexer = Lexer::new(code);
        let tokens = lexer.get_tokens();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let ref_program = CProgram {
            function: CFunction {
                name: "main".to_string(),
                body: CStatement::Return(CExpression::Unary(
                    UnaryOp::BitwiseComplement,
                    Box::new(CExpression::Constant(2)),
                )),
            },
        };

        assert_eq!(program, ref_program)
    }

    #[test]
    fn return_not_neg_2() {
        let code = b"int main(void) {
        return (~((-2)));
        }";

        let mut lexer = Lexer::new(code);
        let tokens = lexer.get_tokens();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let ref_program = CProgram {
            function: CFunction {
                name: "main".to_string(),
                body: CStatement::Return(CExpression::Unary(
                    UnaryOp::BitwiseComplement,
                    Box::new(CExpression::Unary(
                        UnaryOp::Negation,
                        Box::new(CExpression::Constant(2)),
                    )),
                )),
            },
        };

        assert_eq!(program, ref_program)
    }
}
