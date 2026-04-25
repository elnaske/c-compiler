use crate::common::{BinaryOp, Keyword, Operator, VarName};
use crate::lexer::Token;

pub mod c_ast;
use c_ast::*;
pub mod semantic_analysis;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
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

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
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

        Ok(CFunction {
            name,
            body: self.parse_block()?,
        })
    }

    fn parse_block(&mut self) -> Result<CBlock, String> {
        let mut body = Vec::<CBlockItem>::new();
        while let Some(token) = self.peek()
            && *token != Token::CloseBrace
        {
            body.push(self.parse_block_item()?);
        }

        self.expect(Token::CloseBrace)?;

        Ok(CBlock(body))
    }

    fn parse_block_item(&mut self) -> Result<CBlockItem, String> {
        if self.peek() == Some(&Token::Keyword(Keyword::Int)) {
            self.advance(); // only int for now, so can skip
            Ok(CBlockItem::Declaration(self.parse_declaration()?))
        } else {
            Ok(CBlockItem::Statement(self.parse_statement()?))
        }
    }

    fn parse_declaration(&mut self) -> Result<CDeclaration, String> {
        if let Ok(CFactor::Var(var)) = self.parse_factor() {
            let exp = match self.peek() {
                Some(Token::Operator(Operator::Assign)) => {
                    self.advance();
                    Some(self.parse_expression(0)?)
                }
                _ => None,
            };
            self.expect(Token::Semicolon)?;
            Ok(CDeclaration { var, init: exp })
        } else {
            Err("Invalid variable name".to_string())
        }
    }

    fn parse_statement(&mut self) -> Result<CStatement, String> {
        match self.peek() {
            Some(Token::Keyword(Keyword::Return)) => {
                self.advance();
                let expression = self.parse_expression(0)?;

                self.expect(Token::Semicolon)?;
                Ok(CStatement::Return(expression))
            }
            Some(Token::Keyword(Keyword::If)) => {
                self.advance();
                self.expect(Token::OpenParenthesis)?;
                let condition = self.parse_expression(0)?;
                self.expect(Token::CloseParenthesis)?;
                let statement = Box::new(self.parse_statement()?);

                let else_statement = match self.peek() {
                    Some(&Token::Keyword(Keyword::Else)) => {
                        self.advance();
                        Some(Box::new(self.parse_statement()?))
                    }
                    _ => None,
                };
                Ok(CStatement::If(condition, statement, else_statement))
            }
            Some(Token::Keyword(Keyword::Break)) => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(CStatement::Break(None))
            }
            Some(Token::Keyword(Keyword::Continue)) => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(CStatement::Continue(None))
            }
            Some(Token::Keyword(Keyword::While)) => {
                self.advance();
                self.expect(Token::OpenParenthesis)?;
                let cond = self.parse_expression(0)?;
                self.expect(Token::CloseParenthesis)?;
                let body = self.parse_statement()?;
                Ok(CStatement::While(cond, Box::new(body), None))
            }
            Some(Token::Keyword(Keyword::Do)) => {
                self.advance();
                let body = self.parse_statement()?;
                self.expect(Token::Keyword(Keyword::While))?;
                self.expect(Token::OpenParenthesis)?;
                let cond = self.parse_expression(0)?;
                self.expect(Token::CloseParenthesis)?;
                self.expect(Token::Semicolon)?;
                Ok(CStatement::DoWhile(Box::new(body), cond, None))
            }
            Some(Token::Keyword(Keyword::For)) => {
                self.advance();
                self.expect(Token::OpenParenthesis)?;
                let init = {
                    if self.peek() == Some(&Token::Keyword(Keyword::Int)) {
                        self.advance(); // only int for now, so can skip
                        CForInit::InitDecl(self.parse_declaration()?)
                    } else {
                        CForInit::InitExp(self.parse_optional_expression(Token::Semicolon)?)
                    }
                };
                let cond = self.parse_optional_expression(Token::Semicolon)?;
                let post = self.parse_optional_expression(Token::CloseParenthesis)?;
                let body = self.parse_statement()?;
                Ok(CStatement::For(init, cond, post, Box::new(body), None))
            }
            Some(Token::OpenBrace) => {
                self.advance();
                Ok(CStatement::Compound(self.parse_block()?))
            }
            Some(Token::Semicolon) => {
                self.advance();
                Ok(CStatement::Null)
            }
            _ => {
                let expression = self.parse_expression(0)?;

                self.expect(Token::Semicolon)?;
                Ok(CStatement::Expression(expression))
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.take_token() {
            Some(Token::Identifier(s)) => Ok(s.clone()),
            other => Err(format!("Expected identifier, found {:?}", other)),
        }
    }

    fn parse_expression(&mut self, min_precedence: u32) -> Result<CExpression, String> {
        let mut left = CExpression::Factor(Box::new(self.parse_factor()?));
        while let Some(Token::Operator(op)) = self.peek()
            && op.is_binary()
            && op.precedence() >= min_precedence
        {
            let op = op.to_binop().unwrap();
            self.advance();
            match op {
                BinaryOp::Assign => {
                    let right = self.parse_expression(op.precedence())?;
                    left = CExpression::Assign(Box::new(left), Box::new(right));
                }
                BinaryOp::Conditional => {
                    let middle = {
                        let exp = self.parse_expression(0)?;
                        self.expect(Token::Colon)?;
                        exp
                    };
                    let right = self.parse_expression(op.precedence())?;
                    left =
                        CExpression::Conditional(Box::new(left), Box::new(middle), Box::new(right));
                }
                _ => {
                    let right = self.parse_expression(op.precedence() + 1)?;
                    left = CExpression::Binary(op, Box::new(left), Box::new(right));
                }
            }
        }

        Ok(left)
    }

    fn parse_optional_expression(
        &mut self,
        end_token: Token,
    ) -> Result<Option<CExpression>, String> {
        let curr_pos = self.pos;
        let exp = match self.parse_expression(0) {
            Ok(exp) => Some(exp),
            Err(_) => {
                self.pos = curr_pos;
                None
            }
        };
        self.expect(end_token)?;
        Ok(exp)
    }

    fn parse_factor(&mut self) -> Result<CFactor, String> {
        match self.take_token() {
            Some(Token::Constant(i)) => Ok(CFactor::Constant(*i)),
            Some(Token::Operator(Operator::Decrement)) => unimplemented!(),
            Some(Token::Operator(op)) if op.is_unary() => {
                // Not a big fan of how this is handled rn
                Ok(CFactor::Unary(
                    op.to_unop().unwrap(),
                    Box::new(self.parse_factor()?),
                ))
            }
            Some(Token::OpenParenthesis) => {
                let inner_exp = self.parse_expression(0)?;
                self.expect(Token::CloseParenthesis)?;
                Ok(CFactor::Expression(inner_exp))
            }
            Some(Token::Identifier(name)) => Ok(CFactor::Var(CVar {
                name: VarName::from(name.clone()),
                id: None,
            })),
            other => Err(format!("Expected expression, found {:?}", other)),
        }
    }
}
