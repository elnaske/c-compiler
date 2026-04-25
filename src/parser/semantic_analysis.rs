use crate::common::{TempId, VarName};
use crate::parser::c_ast::*;

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct VarMapEntry {
    id: TempId,
    is_from_current_block: bool,
}

fn copy_variable_map(
    variable_map: &HashMap<VarName, VarMapEntry>,
) -> HashMap<VarName, VarMapEntry> {
    let mut new_var_map = variable_map.clone();
    for entry in new_var_map.values_mut() {
        entry.is_from_current_block = false;
    }
    new_var_map
}

pub struct SemanticAnalyzer {
    next_var_id: u32,
}
impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer { next_var_id: 0 }
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

    fn resolve_optional_expression(
        &mut self,
        expression: Option<CExpression>,
        variable_map: &mut HashMap<VarName, VarMapEntry>,
    ) -> Result<Option<CExpression>, String> {
        Ok(match expression {
            Some(e) => Some(self.resolve_expression(e, variable_map)?),
            None => None,
        })
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
            CStatement::If(cond, mut then, else_) => {
                *then = self.resolve_statement(*then, variable_map)?;
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
                // let mut new_var_map = self.copy_variable_map(variable_map);
                Ok(CStatement::Compound(self.resolve_block(
                    block,
                    &mut copy_variable_map(variable_map),
                )?))
            }
            CStatement::While(cond, mut body, label) => {
                let cond = self.resolve_expression(cond, variable_map)?;
                *body = self.resolve_statement(*body, variable_map)?;
                Ok(CStatement::While(cond, body, label))
            }
            CStatement::DoWhile(body, cond, label) => {
                let cond = self.resolve_expression(cond, variable_map)?;
                let body = Box::new(self.resolve_statement(*body, variable_map)?);
                Ok(CStatement::DoWhile(body, cond, label))
            }
            CStatement::For(init, cond, post, mut body, label) => {
                let mut new_var_map = copy_variable_map(variable_map);
                let init = match init {
                    CForInit::InitDecl(dec) => {
                        CForInit::InitDecl(self.resolve_declaration(dec, &mut new_var_map)?)
                    }
                    CForInit::InitExp(exp) => {
                        CForInit::InitExp(self.resolve_optional_expression(exp, &mut new_var_map)?)
                    }
                };
                let cond = self.resolve_optional_expression(cond, &mut new_var_map)?;
                let post = self.resolve_optional_expression(post, &mut new_var_map)?;
                *body = self.resolve_statement(*body, &mut new_var_map)?;

                Ok(CStatement::For(init, cond, post, body, label))
            }
            CStatement::Break(_label) | CStatement::Continue(_label) => Ok(statement),
            CStatement::Null => Ok(statement),
        }
    }
}
