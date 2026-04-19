use crate::common::{BinaryOp, Keyword, Operator};
use crate::lexer::Token;
use std::collections::HashMap;

pub mod c_ast;
use c_ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    variable_map: HashMap<String, String>,
    next_var_id: u32,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            variable_map: HashMap::new(),
            next_var_id: 0,
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

    fn expect_where<F>(&mut self, check_token: F, expected_desc: &str) -> Result<(), String>
    where
        F: Fn(&Token) -> bool,
    {
        let actual = self.take_token();

        match actual {
            Some(token) if check_token(token) => Ok(()),
            _ => Err(format!(
                "Syntax error: expected {}, found '{:?}'",
                expected_desc, actual
            )),
        }
    }

    fn create_unique_var(&mut self, var_name: &str) -> String {
        let unique_name = format!("{}.{}", var_name, self.next_var_id);
        self.next_var_id += 1;
        unique_name
    }

    fn parse_function(&mut self) -> Result<CFunction, String> {
        self.expect(Token::Keyword(Keyword::Int))?;

        let name = self.parse_identifier()?;

        self.expect(Token::OpenParenthesis)?;
        self.expect(Token::Keyword(Keyword::Void))?;
        self.expect(Token::CloseParenthesis)?;
        self.expect(Token::OpenBrace)?;

        // TODO: iterator
        let mut body = Vec::<CBlockItem>::new();
        while let Some(token) = self.peek()
            && *token != Token::CloseBrace
        {
            body.push(self.parse_block_item()?);
        }

        self.expect(Token::CloseBrace)?;

        Ok(CFunction { name, body })
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
        if let Ok(CFactor::Var(name)) = self.parse_factor() {
            let exp = match self.peek() {
                Some(Token::Operator(Operator::Assign)) => {
                    self.advance();
                    Some(self.parse_expression(0)?)
                }
                _ => None,
            };
            self.expect(Token::Semicolon)?;
            Ok(CDeclaration(name, exp))
        } else {
            Err("Invalid variable name".to_string())
        }
    }

    fn parse_statement(&mut self) -> Result<CStatement, String> {
        if self.peek() == Some(&Token::Keyword(Keyword::Return)) {
            self.advance();
        }

        match self.peek() {
            Some(Token::Semicolon) => {
                self.advance();
                Ok(CStatement::Null)
            }
            _ => {
                let expression = self.parse_expression(0)?;

                self.expect(Token::Semicolon)?;
                Ok(CStatement::Return(expression))
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
            if op == BinaryOp::Assign {
                let right = self.parse_expression(op.precedence())?;
                left = CExpression::Assign(Box::new(left), Box::new(right));
            } else {
                let right = self.parse_expression(op.precedence() + 1)?;
                left = CExpression::Binary(op, Box::new(left), Box::new(right));
            }
        }

        Ok(left)
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
            Some(Token::Identifier(name)) => {
                // TODO: borrow here
                Ok(CFactor::Var(name.clone()))
            }
            other => Err(format!("Expected expression, found {:?}", other)),
        }
    }

    // TODO: do parsing and resolution in one pass
    // TODO: remove unwrap()
    pub fn resolve_variables(&mut self, program: CProgram) -> Result<CProgram, String> {
        Ok(CProgram {
            function: CFunction {
                name: program.function.name,
                body: program
                    .function
                    .body
                    .into_iter()
                    .map(|block| match block {
                        CBlockItem::Declaration(dec) => {
                            CBlockItem::Declaration(self.resolve_declaration(dec).unwrap())
                        }
                        CBlockItem::Statement(stmnt) => {
                            CBlockItem::Statement(self.resolve_statement(stmnt).unwrap())
                        }
                    })
                    .collect(),
            },
        })
    }

    fn resolve_declaration(&mut self, declaration: CDeclaration) -> Result<CDeclaration, String> {
        let (name, mut exp) = (declaration.0, declaration.1);
        if self.variable_map.contains_key(&name) {
            return Err(format!("Variable `{}` is declared twice", name));
        }
        let unique_name = self.create_unique_var(&name);
        self.variable_map.insert(name, unique_name.clone());
        if let Some(e) = exp {
            exp = Some(self.resolve_expression(e)?);
        }
        Ok(CDeclaration(unique_name, exp))
    }

    // TODO: resolve in place instead of allocating new pointers
    fn resolve_expression(&mut self, expression: CExpression) -> Result<CExpression, String> {
        match expression {
            CExpression::Assign(left, right) => match *left {
                CExpression::Factor(ref f) if matches!(**f, CFactor::Var(_)) => {
                    let left = self.resolve_expression(*left)?;
                    let right = self.resolve_expression(*right)?;
                    Ok(CExpression::Assign(Box::new(left), Box::new(right)))
                }
                other => Err(format!("Invalid lvalue `{}`", other)),
            },
            CExpression::Binary(op, left, right) => {
                let left = self.resolve_expression(*left)?;
                let right = self.resolve_expression(*right)?;
                Ok(CExpression::Binary(op, Box::new(left), Box::new(right)))
            }
            CExpression::Factor(f) => {
                let f = self.resolve_factor(*f)?;
                Ok(CExpression::Factor(Box::new(f)))
            }
        }
    }

    fn resolve_factor(&mut self, factor: CFactor) -> Result<CFactor, String> {
        match factor {
            CFactor::Var(ref name) => match self.variable_map.get(name) {
                Some(unique_name) => Ok(CFactor::Var(unique_name.clone())),
                None => Err(format!("Undeclared variable `{}`", &name)),
            },
            CFactor::Unary(op, f2) => {
                let f2 = self.resolve_factor(*f2)?;
                Ok(CFactor::Unary(op, Box::new(f2)))
            }
            CFactor::Expression(exp) => Ok(CFactor::Expression(self.resolve_expression(exp)?)),
            CFactor::Constant(_) => Ok(factor),
        }
    }

    fn resolve_statement(&mut self, statement: CStatement) -> Result<CStatement, String> {
        match statement {
            CStatement::Return(exp) => Ok(CStatement::Return(self.resolve_expression(exp)?)),
            CStatement::Expression(exp) => {
                Ok(CStatement::Expression(self.resolve_expression(exp)?))
            }
            CStatement::Null => Ok(CStatement::Null),
        }
    }
}
