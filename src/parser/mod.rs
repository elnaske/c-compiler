use crate::common::{BinaryOp, Keyword, Operator};
use crate::lexer::Token;

pub mod c_ast;
use c_ast::*;
pub mod semantic_analysis;
pub mod type_checker;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<CProgram, String> {
        let mut functions = Vec::<CFnDecl>::new();
        while let Some(Token::Keyword(Keyword::Int)) = self.peek() {
            self.expect(Token::Keyword(Keyword::Int))?;
            functions.push(self.parse_fn_decl()?);
        }

        if self.pos < self.tokens.len() {
            Err("leftover tokens after program".to_string())
        } else {
            Ok(CProgram { functions })
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

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
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

    fn parse_fn_decl(&mut self) -> Result<CFnDecl, String> {
        let name = self.parse_identifier()?;

        self.expect(Token::OpenParenthesis)?;
        let params = self.parse_params()?;
        self.expect(Token::CloseParenthesis)?;

        let body = {
            match self.peek() {
                Some(Token::OpenBrace) => {
                    self.advance();
                    Some(self.parse_block()?)
                }
                _ => {
                    self.expect(Token::Semicolon)?;
                    None
                }
            }
        };

        Ok(CFnDecl { name, params, body })
    }

    fn parse_params(&mut self) -> Result<Vec<CParam>, String> {
        if self.peek() == Some(&Token::Keyword(Keyword::Void)) {
            self.advance();
            // return Ok(vec![CParam {
            //     keyword: Keyword::Void,
            //     name: None,
            // }]);
            return Ok(vec![]);
        }

        let mut params = Vec::<CParam>::new();

        loop {
            self.expect(Token::Keyword(Keyword::Int))?;
            match self.take_token() {
                Some(Token::Identifier(name)) => {
                    params.push(CParam {
                        keyword: Keyword::Int,
                        name: Some(name.clone()),
                        id: None,
                    });
                    if self.peek() == Some(&Token::CloseParenthesis) {
                        break;
                    }
                    self.expect(Token::Comma)?;
                }
                _ => return Err("Parameter without identifier".to_string()),
            }
        }
        Ok(params)
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
            Ok(CBlockItem::Declaration(self.parse_declaration()?))
        } else {
            Ok(CBlockItem::Statement(self.parse_statement()?))
        }
    }

    fn parse_declaration(&mut self) -> Result<CDeclaration, String> {
        self.expect(Token::Keyword(Keyword::Int))?;

        // check token after identifier
        match self.peek_next() {
            Some(&Token::OpenParenthesis) => Ok(CDeclaration::FnDecl(self.parse_fn_decl()?)),
            _ => Ok(CDeclaration::VarDecl(self.parse_var_declaration()?)),
        }
    }

    fn parse_var_declaration(&mut self) -> Result<CVarDecl, String> {
        match self.parse_factor() {
            Ok(CFactor::Var(var)) => {
                let exp = match self.peek() {
                    Some(Token::Operator(Operator::Assign)) => {
                        self.advance();
                        Some(self.parse_expression(0)?)
                    }
                    _ => None,
                };
                self.expect(Token::Semicolon)?;
                Ok(CVarDecl { var, init: exp })
            }
            other => Err(format!("Invalid variable name: `{:?}`", other)),
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
                        self.expect(Token::Keyword(Keyword::Int))?;
                        CForInit::InitDecl(self.parse_var_declaration()?)
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
            Some(Token::Operator(op)) if op.is_unary() => Ok(CFactor::Unary(
                op.to_unop().unwrap(),
                Box::new(self.parse_factor()?),
            )),
            Some(Token::OpenParenthesis) => {
                let inner_exp = self.parse_expression(0)?;
                self.expect(Token::CloseParenthesis)?;
                Ok(CFactor::Expression(inner_exp))
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.parse_var_or_fn_call(name)
            }
            other => Err(format!("Expected expression, found {:?}", other)),
        }
    }

    fn parse_var_or_fn_call(&mut self, name: String) -> Result<CFactor, String> {
        match self.peek() {
            Some(Token::OpenParenthesis) => {
                self.advance();
                let curr_pos = self.pos;
                let args = match self.parse_args() {
                    Ok(args) => args,
                    Err(_) => {
                        self.pos = curr_pos;
                        vec![]
                    }
                };

                self.expect(Token::CloseParenthesis)?;
                Ok(CFactor::FunctionCall(name, args))
            }
            _ => Ok(CFactor::Var(CVar { name, id: None })),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<CExpression>, String> {
        let mut args = Vec::<CExpression>::new();

        loop {
            args.push(self.parse_expression(0)?);
            if self.peek() == Some(&Token::CloseParenthesis) {
                break;
            }
            self.expect(Token::Comma)?;
        }

        Ok(args)
    }
}
