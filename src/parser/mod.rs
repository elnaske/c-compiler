use crate::common::{BinaryOp, Keyword, Operator, TempId, VarName};
use crate::lexer::Token;
use std::collections::HashMap;

pub mod c_ast;
use c_ast::*;

#[derive(Debug, Clone)]
struct VarMapEntry {
    id: TempId,
    is_from_current_block: bool,
}

pub struct Parser {
    tokens: Vec<Token>,
    //variable_map: HashMap<VarName, TempId>,
    next_var_id: u32,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            // variable_map: HashMap::new(),
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

    fn create_unique_var(&mut self) -> TempId {
        let unique_var = TempId(self.next_var_id);
        self.next_var_id += 1;
        unique_var
    }

    pub fn get_next_var_id(self) -> u32 {
        self.next_var_id
    }

    pub fn resolve_variables(&mut self, program: CProgram) -> Result<CProgram, String> {
        let mut var_map = HashMap::<VarName, VarMapEntry>::new();
        Ok(CProgram {
            function: CFunction {
                name: program.function.name,
                body: self.resolve_block(program.function.body, &mut var_map)?,
            },
        })
    }

    fn resolve_block(
        &mut self,
        block: CBlock,
        variable_map: &mut HashMap<VarName, VarMapEntry>,
    ) -> Result<CBlock, String> {
        Ok(CBlock(
            block
                .0
                .into_iter()
                .map(|block| match block {
                    CBlockItem::Declaration(dec) => CBlockItem::Declaration(
                        self.resolve_declaration(dec, variable_map).unwrap(),
                    ),
                    CBlockItem::Statement(stmnt) => {
                        CBlockItem::Statement(self.resolve_statement(stmnt, variable_map).unwrap())
                    }
                })
                .collect(),
        ))
    }

    fn resolve_declaration(
        &mut self,
        declaration: CDeclaration,
        variable_map: &mut HashMap<VarName, VarMapEntry>,
    ) -> Result<CDeclaration, String> {
        let (var, mut exp) = (declaration.var, declaration.init);
        if let Some(entry) = variable_map.get(&var.name)
            && entry.is_from_current_block
        {
            return Err(format!("Variable `{}` is declared twice", var.name));
        }
        let id = self.create_unique_var();
        variable_map.insert(
            var.name.clone(),
            VarMapEntry {
                id,
                is_from_current_block: true,
            },
        );
        if let Some(e) = exp {
            exp = Some(self.resolve_expression(e, variable_map)?);
        }
        let unique_var = CVar {
            name: var.name,
            id: Some(id),
        };
        Ok(CDeclaration {
            var: unique_var,
            init: exp,
        })
    }

    // TODO: resolve in place instead of allocating new pointers and cloning (iter_mut()?)
    fn resolve_expression(
        &mut self,
        expression: CExpression,
        variable_map: &mut HashMap<VarName, VarMapEntry>,
    ) -> Result<CExpression, String> {
        match expression {
            CExpression::Assign(left, right) => match *left {
                CExpression::Factor(ref f) if matches!(**f, CFactor::Var(_)) => {
                    let left = self.resolve_expression(*left, variable_map)?;
                    let right = self.resolve_expression(*right, variable_map)?;
                    Ok(CExpression::Assign(Box::new(left), Box::new(right)))
                }
                other => Err(format!("Invalid lvalue `{}`", other)),
            },
            CExpression::Binary(op, left, right) => {
                let left = self.resolve_expression(*left, variable_map)?;
                let right = self.resolve_expression(*right, variable_map)?;
                Ok(CExpression::Binary(op, Box::new(left), Box::new(right)))
            }
            CExpression::Factor(f) => {
                let f = self.resolve_factor(*f, variable_map)?;
                Ok(CExpression::Factor(Box::new(f)))
            }
            CExpression::Conditional(cond, exp1, exp2) => {
                let cond = self.resolve_expression(*cond, variable_map)?;
                let exp1 = self.resolve_expression(*exp1, variable_map)?;
                let exp2 = self.resolve_expression(*exp2, variable_map)?;
                Ok(CExpression::Conditional(
                    Box::new(cond),
                    Box::new(exp1),
                    Box::new(exp2),
                ))
            }
        }
    }

    fn resolve_factor(
        &mut self,
        factor: CFactor,
        variable_map: &mut HashMap<VarName, VarMapEntry>,
    ) -> Result<CFactor, String> {
        match factor {
            CFactor::Var(ref var) => match variable_map.get(&var.name) {
                Some(entry) => Ok(CFactor::Var(CVar {
                    name: var.name.clone(),
                    id: Some(entry.id),
                })),
                None => Err(format!("Undeclared variable `{}`", var.name)),
            },
            CFactor::Unary(op, f2) => {
                let f2 = self.resolve_factor(*f2, variable_map)?;
                Ok(CFactor::Unary(op, Box::new(f2)))
            }
            CFactor::Expression(exp) => Ok(CFactor::Expression(
                self.resolve_expression(exp, variable_map)?,
            )),
            CFactor::Constant(_) => Ok(factor),
        }
    }

    fn resolve_statement(
        &mut self,
        statement: CStatement,
        variable_map: &mut HashMap<VarName, VarMapEntry>,
    ) -> Result<CStatement, String> {
        match statement {
            CStatement::Return(exp) => Ok(CStatement::Return(
                self.resolve_expression(exp, variable_map)?,
            )),
            CStatement::Expression(exp) => Ok(CStatement::Expression(
                self.resolve_expression(exp, variable_map)?,
            )),
            CStatement::If(cond, then, else_) => {
                let then = Box::new(self.resolve_statement(*then, variable_map)?);
                let else_ = match else_ {
                    Some(else_stmnt) => {
                        Some(Box::new(self.resolve_statement(*else_stmnt, variable_map)?))
                    }
                    None => None,
                };
                Ok(CStatement::If(
                    self.resolve_expression(cond, variable_map)?,
                    then,
                    else_,
                ))
            }
            CStatement::Compound(block) => {
                let mut new_var_map = variable_map.clone();
                for k in variable_map.keys() {
                    let entry = new_var_map.get_mut(k).unwrap();
                    entry.is_from_current_block = false;
                }
                Ok(CStatement::Compound(
                    self.resolve_block(block, &mut new_var_map)?,
                ))
            }
            CStatement::Null => Ok(CStatement::Null),
        }
    }
}
