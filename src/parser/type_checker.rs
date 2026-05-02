use std::collections::HashMap;

use crate::parser::c_ast::*;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Type {
    Int,
    FnType { param_count: u32 },
}

#[derive(Debug)]
enum SymbolKind {
    Fn { defined: bool },
    Var,
}

#[derive(Debug)]
pub struct SymbolEntry {
    kind: SymbolKind,
    type_: Type,
}

#[derive(Debug)]
pub struct TypeChecker {
    pub symbols: HashMap<String, SymbolEntry>,
}
impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbols: HashMap::<String, SymbolEntry>::new(),
        }
    }

    pub fn type_check(&mut self, program: &CProgram) -> Result<(), String> {
        for f in &program.functions {
            self.check_fn_decl(f)?;
        }

        Ok(())
    }

    fn check_fn_decl(&mut self, decl: &CFnDecl) -> Result<(), String> {
        let fn_type = Type::FnType {
            param_count: decl.params.len() as u32,
        };
        let has_body = decl.body.is_some();
        let mut already_defined = false;

        let name = format!("{}.0", decl.name);
        if let Some(old_decl) = self.symbols.get(&name) {
            if old_decl.type_ != fn_type {
                return Err(format!(
                    "Found incompatible function declarations for `{}`: `{:?}` and `{:?}`",
                    decl.name, old_decl.type_, fn_type,
                ));
            }
            if let SymbolKind::Fn { defined: def } = old_decl.kind {
                already_defined = def;
                if already_defined && has_body {
                    return Err(format!(
                        "Function `{}()` is defined more than once",
                        decl.name
                    ));
                }
            };
        }

        self.symbols.insert(
            format!("{}.0", decl.name),
            SymbolEntry {
                kind: SymbolKind::Fn {
                    defined: already_defined || has_body,
                },
                type_: fn_type,
            },
        );

        if let Some(body) = &decl.body {
            for param in &decl.params {
                // only void will be None
                if let Some(name) = &param.name {
                    self.symbols.insert(
                        name.clone(),
                        SymbolEntry {
                            kind: SymbolKind::Var,
                            type_: Type::Int,
                        },
                    );
                }
            }
            self.check_block(body)?;
        }

        Ok(())
    }

    fn check_var_decl(&mut self, var_decl: &CVarDecl) -> Result<(), String> {
        self.symbols.insert(
            var_decl.var.to_string(),
            SymbolEntry {
                kind: SymbolKind::Var,
                type_: Type::Int,
            },
        );
        if let Some(exp) = &var_decl.init {
            self.check_exp(exp)?;
        }
        Ok(())
    }

    fn check_block(&mut self, block: &CBlock) -> Result<(), String> {
        for item in &block.0 {
            match item {
                CBlockItem::Statement(stmnt) => self.check_statement(stmnt),
                CBlockItem::Declaration(dec) => match dec {
                    CDeclaration::FnDecl(fdec) => self.check_fn_decl(fdec),
                    CDeclaration::VarDecl(vdec) => self.check_var_decl(vdec),
                },
            }?
        }
        Ok(())
    }

    fn check_statement(&mut self, stmnt: &CStatement) -> Result<(), String> {
        match stmnt {
            CStatement::Return(exp) | CStatement::Expression(exp) => self.check_exp(exp)?,
            CStatement::If { cond, then, else_ } => {
                self.check_exp(cond)?;
                self.check_statement(then)?;
                if let Some(s) = else_ {
                    self.check_statement(s)?;
                }
            }
            CStatement::While {
                cond,
                body,
                label: _,
            }
            | CStatement::DoWhile {
                body,
                cond,
                label: _,
            } => {
                self.check_exp(cond)?;
                self.check_statement(body)?
            }
            CStatement::For {
                init,
                cond,
                post,
                body: _,
                label: _,
            } => {
                match init {
                    CForInit::InitDecl(dec) => self.check_var_decl(dec)?,
                    CForInit::InitExp(exp) if exp.is_some() => {
                        self.check_exp(exp.as_ref().unwrap())?
                    }
                    _ => (),
                }

                if let Some(e) = cond {
                    self.check_exp(e)?;
                }
                if let Some(e) = post {
                    self.check_exp(e)?;
                }
            }
            CStatement::Compound(block) => self.check_block(block)?,
            _ => (),
        }
        Ok(())
    }

    fn check_exp(&mut self, exp: &CExpression) -> Result<(), String> {
        match exp {
            CExpression::Factor(f) => self.check_factor(f)?,
            CExpression::Binary(_, exp1, exp2) | CExpression::Assign(exp1, exp2) => {
                self.check_exp(exp1)?;
                self.check_exp(exp2)?;
            }
            CExpression::Conditional { cond, then, else_ } => {
                self.check_exp(cond)?;
                self.check_exp(then)?;
                self.check_exp(else_)?;
            }
        }
        Ok(())
    }

    fn check_factor(&mut self, factor: &CFactor) -> Result<(), String> {
        match factor {
            CFactor::FunctionCall(f, args) => {
                let fn_type = match self.symbols.get(f) {
                    Some(entry) => entry.type_,
                    None => {
                        return Err(format!(
                            "`{}()` is called before it is declared; This should have been taken care of during identifier resolution",
                            f
                        ));
                    }
                };
                if let Type::FnType {
                    param_count: n_params,
                } = fn_type
                {
                    if n_params != args.len() as u32 {
                        return Err(format!(
                            "Function `{}()` called with the wrong number of arguments",
                            f
                        ));
                    }
                } else {
                    return Err("Variable used as function name".to_string());
                }

                for arg in args {
                    self.check_exp(arg)?;
                }
            }
            // CFactor::Var(var) => match self.symbols.get(&var.name) {
            CFactor::Var(var) => match self.symbols.get(&var.to_string()) {
                Some(entry) => {
                    if entry.type_ != Type::Int {
                        return Err("Function name used as variable".to_string());
                    }
                }
                None => {
                    eprintln!("{:#?}", self.symbols);
                    return Err(format!(
                        "Variable `{}` is called before it is declared; This should have been taken care of during identifier resolution.",
                        var
                    ));
                }
            },
            CFactor::Expression(exp) => self.check_exp(exp)?,
            CFactor::Unary(_, f) => self.check_factor(f)?,
            CFactor::Constant(_) => (),
        }
        Ok(())
    }
}
