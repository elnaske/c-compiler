use crate::common::TempId;
use crate::ir::ir_ast::{Label, LabelKind};
use crate::parser::c_ast::*;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
struct IdentifierMapEntry {
    id: TempId,
    is_from_current_scope: bool,
    has_linkage: bool,
}

#[derive(Debug, Clone)]
struct IdentifierMap(HashMap<String, IdentifierMapEntry>);
impl IdentifierMap {
    fn new() -> Self {
        IdentifierMap(HashMap::new())
    }
}
impl Deref for IdentifierMap {
    type Target = HashMap<String, IdentifierMapEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for IdentifierMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn copy_variable_map(variable_map: &IdentifierMap) -> IdentifierMap {
    let mut new_var_map = variable_map.clone();
    for entry in new_var_map.values_mut() {
        entry.is_from_current_scope = false;
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
            next_var_id: 1, // id 0 is for functions (kinda hacky but it works for now)
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
        Ok(CProgram {
            functions: program
                .functions
                .into_iter()
                .map(|f| CFnDecl {
                    name: f.name,
                    params: f.params,
                    body: f.body.map(|b| self.label_block(b, None).unwrap()),
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
                Some(label) => Ok(CStatement::Break(Some(Label {
                    kind: LabelKind::Break,
                    id: label.id,
                }))),
                None => Err("Break statement outside of loop".to_string()),
            },
            CStatement::Continue(_) => match curr_label {
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

    pub fn resolve_variables(&mut self, program: &mut CProgram) -> Result<(), String> {
        let mut var_map = IdentifierMap::new();

        for function in &mut program.functions {
            self.resolve_fn_declaration(function, &mut var_map)?;
        }

        Ok(())
    }

    fn resolve_fn_declaration(
        &mut self,
        function: &mut CFnDecl,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        if let Some(entry) = variable_map.get(&function.name)
            && entry.is_from_current_scope
            && !entry.has_linkage
        {
            return Err(format!("Function `{}` is declared twice.", function.name));
        }

        variable_map.insert(
            function.name.clone(),
            IdentifierMapEntry {
                id: TempId(0), // id 0 reserved for functions, variables start at 1
                is_from_current_scope: true,
                has_linkage: true,
            },
        );

        let mut inner_map = copy_variable_map(variable_map);
        for param in &mut function.params {
            self.resolve_param(param, &mut inner_map)?;
        }

        if let Some(b) = &mut function.body {
            self.resolve_block(b, &mut inner_map)?;
        }

        Ok(())
    }

    fn resolve_block(
        &mut self,
        block: &mut CBlock,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        for item in &mut block.0 {
            match item {
                CBlockItem::Declaration(dec) => match dec {
                    CDeclaration::VarDecl(vdec) => {
                        self.resolve_var_declaration(vdec, variable_map)?
                    }

                    CDeclaration::FnDecl(fdec) => {
                        // nested function definitions are not allowed, but function declarations are
                        match fdec.body {
                            None => self.resolve_fn_declaration(fdec, variable_map)?,
                            Some(_) => {
                                return Err(format!("Nested function definition: `{}`", fdec.name));
                            }
                        };
                    }
                },
                CBlockItem::Statement(stmnt) => {
                    self.resolve_statement(stmnt, variable_map)?;
                }
            }
        }
        Ok(())
    }

    fn resolve_param(
        &mut self,
        param: &mut CParam,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        if let Some(ref name) = param.name {
            let id = self.get_var_or_param_id(name, variable_map)?;
            param.name = Some(format!("{}.{}", name, id.0));
            param.id = Some(id);
        }
        Ok(())
    }

    // TODO: rename variable_map to id_map
    fn resolve_var_declaration(
        &mut self,
        var_decl: &mut CVarDecl,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        var_decl.var.id = Some(self.get_var_or_param_id(&var_decl.var.name, variable_map)?);

        self.resolve_optional_expression(&mut var_decl.init, variable_map)?;
        Ok(())
    }

    fn get_var_or_param_id(
        &mut self,
        name: &str,
        variable_map: &mut IdentifierMap,
    ) -> Result<TempId, String> {
        if let Some(entry) = variable_map.get(name)
            && entry.is_from_current_scope
        {
            return Err(format!("`{}` is declared twice", name));
        }
        let id = self.create_unique_var();
        variable_map.insert(
            name.to_string(),
            IdentifierMapEntry {
                id,
                is_from_current_scope: true,
                has_linkage: false,
            },
        );

        Ok(id)
    }

    fn resolve_expression(
        &mut self,
        expression: &mut CExpression,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        match expression {
            CExpression::Assign(left, right) => {
                self.resolve_assignment(left, right, variable_map)?;
            }
            CExpression::Binary(_, left, right) => {
                self.resolve_expression(left, variable_map)?;
                self.resolve_expression(right, variable_map)?;
            }
            CExpression::Factor(f) => {
                self.resolve_factor(f, variable_map)?;
            }
            CExpression::Conditional(cond, exp1, exp2) => {
                self.resolve_expression(cond, variable_map)?;
                self.resolve_expression(exp1, variable_map)?;
                self.resolve_expression(exp2, variable_map)?;
            }
        }
        Ok(())
    }

    fn resolve_assignment(
        &mut self,
        lval: &mut CExpression,
        rval: &mut CExpression,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        match lval {
            CExpression::Factor(f) if matches!(**f, CFactor::Var(_)) => {
                self.resolve_expression(lval, variable_map)?;
                self.resolve_expression(rval, variable_map)?;
            }
            other => return Err(format!("Invalid lvalue `{}`", other)),
        }
        Ok(())
    }

    fn resolve_optional_expression(
        &mut self,
        expression: &mut Option<CExpression>,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        if let Some(e) = expression {
            self.resolve_expression(e, variable_map)?
        }
        Ok(())
    }

    fn resolve_factor(
        &mut self,
        factor: &mut CFactor,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        match factor {
            CFactor::Var(var) => match variable_map.get(&var.name) {
                Some(entry) => {
                    var.id = Some(entry.id);
                }
                None => return Err(format!("Undeclared variable `{}`", var.name)),
            },
            CFactor::FunctionCall(name, args) => match variable_map.get(name) {
                Some(entry) => {
                    *name = format!("{}.{}", name, entry.id.0);
                    for arg in args {
                        self.resolve_expression(arg, variable_map)?
                    }
                }
                None => return Err(format!("Undeclared function `{}`", name)),
            },
            CFactor::Unary(_, f2) => {
                self.resolve_factor(f2, variable_map)?;
            }
            CFactor::Expression(exp) => self.resolve_expression(exp, variable_map)?,
            CFactor::Constant(_) => (),
        }
        Ok(())
    }

    fn resolve_statement(
        &mut self,
        statement: &mut CStatement,
        variable_map: &mut IdentifierMap,
    ) -> Result<(), String> {
        match statement {
            CStatement::Return(exp) | CStatement::Expression(exp) => {
                self.resolve_expression(exp, variable_map)?
            }
            CStatement::If(cond, then, else_) => {
                self.resolve_statement(then, variable_map)?;
                if let Some(stmnt) = else_ {
                    self.resolve_statement(stmnt, variable_map)?;
                }
                self.resolve_expression(cond, variable_map)?;
            }
            CStatement::Compound(block) => {
                self.resolve_block(block, &mut copy_variable_map(variable_map))?
            }
            CStatement::While(cond, body, _) | CStatement::DoWhile(body, cond, _) => {
                self.resolve_expression(cond, variable_map)?;
                self.resolve_statement(body, variable_map)?;
            }
            CStatement::For(init, cond, post, body, _) => {
                let mut new_var_map = copy_variable_map(variable_map);
                match init {
                    CForInit::InitDecl(dec) => {
                        self.resolve_var_declaration(dec, &mut new_var_map)?
                    }
                    CForInit::InitExp(exp) => {
                        self.resolve_optional_expression(exp, &mut new_var_map)?
                    }
                }
                self.resolve_optional_expression(cond, &mut new_var_map)?;
                self.resolve_optional_expression(post, &mut new_var_map)?;
                self.resolve_statement(body, &mut new_var_map)?;
            }
            CStatement::Break(_) | CStatement::Continue(_) | CStatement::Null => (),
        }
        Ok(())
    }
}
