use crate::common::TempId;
use crate::ir::ir_ast::{Label, LabelKind};
use crate::parser::c_ast::*;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
struct VarMapEntry {
    id: TempId,
    is_from_current_block: bool,
}

#[derive(Debug, Clone)]
struct VarMap(HashMap<String, VarMapEntry>);
impl VarMap {
    fn new() -> Self {
        VarMap(HashMap::new())
    }
}
impl Deref for VarMap {
    type Target = HashMap<String, VarMapEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for VarMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn copy_variable_map(variable_map: &VarMap) -> VarMap {
    let mut new_var_map = variable_map.clone();
    for entry in new_var_map.values_mut() {
        entry.is_from_current_block = false;
    }
    new_var_map
}

pub struct SemanticAnalyzer {
    next_var_id: u32,
    next_label_id: u32,
}
impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            next_var_id: 0,
            next_label_id: 0,
        }
    }

    fn create_unique_var(&mut self) -> TempId {
        let unique_var = TempId(self.next_var_id);
        self.next_var_id += 1;
        unique_var
    }

    fn create_jump_label(&mut self, kind: LabelKind) -> Label {
        let id = self.next_label_id;
        self.next_label_id += 1;
        Label { kind, id }
    }

    pub fn get_next_var_id(&self) -> u32 {
        self.next_var_id
    }

    pub fn get_next_label_id(&self) -> u32 {
        self.next_label_id
    }

    // TODO: label in place
    pub fn label_loops(&mut self, program: CProgram) -> Result<CProgram, String> {
        // Ok(CProgram {
        //     function: CFnDecl {
        //         name: program.functions.name,
        //         body: self.label_block(program.function.body, None)?,
        //     },
        // })

        // TODO: remove unwrap()
        Ok(CProgram {
            functions: program
                .functions
                .into_iter()
                .map(|f| CFnDecl {
                    name: f.name,
                    params: f.params,
                    body: match f.body {
                        Some(b) => Some(self.label_block(b, None).unwrap()),
                        None => None,
                    },
                })
                .collect(),
        })
    }

    fn label_block(&mut self, block: CBlock, curr_label: Option<Label>) -> Result<CBlock, String> {
        Ok(CBlock(
            block
                .0
                .into_iter()
                .map(|item| match item {
                    CBlockItem::Statement(stmnt) => {
                        CBlockItem::Statement(self.label_statement(stmnt, curr_label).unwrap())
                    }
                    _ => item,
                })
                .collect(),
        ))
    }

    fn label_statement(
        &mut self,
        statement: CStatement,
        curr_label: Option<Label>,
    ) -> Result<CStatement, String> {
        match statement {
            CStatement::Break(_) => match curr_label {
                // Some(_) => Ok(CStatement::Break(curr_label)),
                Some(label) => Ok(CStatement::Break(Some(Label {
                    kind: LabelKind::Break,
                    id: label.id,
                }))),
                None => Err("Break statement outside of loop".to_string()),
            },
            CStatement::Continue(_) => match curr_label {
                // Some(_) => Ok(CStatement::Continue(curr_label)),
                Some(label) => Ok(CStatement::Continue(Some(Label {
                    kind: LabelKind::Continue,
                    id: label.id,
                }))),
                None => Err("Continue statement outside of loop".to_string()),
            },
            CStatement::While(cond, mut body, _label) => {
                let new_label = Some(self.create_jump_label(LabelKind::LoopStart));
                *body = self.label_statement(*body, new_label)?;
                Ok(CStatement::While(cond, body, new_label))
            }
            CStatement::DoWhile(mut body, cond, _label) => {
                let new_label = Some(self.create_jump_label(LabelKind::LoopStart));
                *body = self.label_statement(*body, new_label)?;
                Ok(CStatement::DoWhile(body, cond, new_label))
            }
            CStatement::For(init, cond, post, mut body, _label) => {
                let new_label = Some(self.create_jump_label(LabelKind::LoopStart));
                *body = self.label_statement(*body, new_label)?;
                Ok(CStatement::For(init, cond, post, body, new_label))
            }
            CStatement::If(cond, mut then, mut else_) => {
                *then = self.label_statement(*then, curr_label)?;
                if let Some(mut stmnt) = else_ {
                    *stmnt = self.label_statement(*stmnt, curr_label)?;
                    else_ = Some(stmnt)
                }
                Ok(CStatement::If(cond, then, else_))
            }
            CStatement::Compound(block) => {
                Ok(CStatement::Compound(self.label_block(block, curr_label)?))
            }
            _ => Ok(statement),
        }
    }

    pub fn resolve_variables(&mut self, program: CProgram) -> Result<CProgram, String> {
        // let mut var_map = VarMap::new();
        // Ok(CProgram {
        //     function: CFnDecl {
        //         name: program.function.name,
        //         body: self.resolve_block(program.function.body, &mut var_map)?,
        //     },
        // })

        // TODO: not sure if this is in the right place ...
        let mut var_map = VarMap::new();

        // TODO: remove unwrap()
        Ok(CProgram {
            functions: program
                .functions
                .into_iter()
                .map(|f| self.resolve_function(f, &mut var_map).unwrap())
                .collect(),
        })
    }

    fn resolve_function(
        &mut self,
        function: CFnDecl,
        variable_map: &mut VarMap,
    ) -> Result<CFnDecl, String> {
        Ok(match function.body {
            Some(b) => CFnDecl {
                name: function.name,
                params: function.params,
                body: Some(self.resolve_block(b, variable_map)?),
            },
            None => function,
        })
    }

    fn resolve_block(
        &mut self,
        block: CBlock,
        variable_map: &mut VarMap,
    ) -> Result<CBlock, String> {
        Ok(CBlock(
            block
                .0
                .into_iter()
                .map(|block| match block {
                    CBlockItem::Declaration(dec) => CBlockItem::Declaration(match dec {
                        CDeclaration::VarDecl(vdec) => CDeclaration::VarDecl(
                            self.resolve_var_declaration(vdec, variable_map).unwrap(),
                        ),
                        CDeclaration::FnDecl(fdec) => CDeclaration::FnDecl(
                            self.resolve_function(fdec, &mut VarMap::new()).unwrap(),
                        ),
                    }),
                    CBlockItem::Statement(stmnt) => {
                        CBlockItem::Statement(self.resolve_statement(stmnt, variable_map).unwrap())
                    }
                })
                .collect(),
        ))
    }

    fn resolve_var_declaration(
        &mut self,
        var_decl: CVarDecl,
        variable_map: &mut VarMap,
    ) -> Result<CVarDecl, String> {
        let (var, mut exp) = (var_decl.var, var_decl.init);
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
        Ok(CVarDecl {
            var: unique_var,
            init: exp,
        })
    }

    // TODO: resolve in place instead of allocating new pointers and cloning (iter_mut()?)
    fn resolve_expression(
        &mut self,
        expression: CExpression,
        variable_map: &mut VarMap,
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
        variable_map: &mut VarMap,
    ) -> Result<Option<CExpression>, String> {
        Ok(match expression {
            Some(e) => Some(self.resolve_expression(e, variable_map)?),
            None => None,
        })
    }

    fn resolve_factor(
        &mut self,
        factor: CFactor,
        variable_map: &mut VarMap,
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
            CFactor::FunctionCall(name, body) => {
                todo!()
            }
        }
    }

    fn resolve_statement(
        &mut self,
        statement: CStatement,
        variable_map: &mut VarMap,
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
                        CForInit::InitDecl(self.resolve_var_declaration(dec, &mut new_var_map)?)
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
