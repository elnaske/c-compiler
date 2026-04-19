use std::fmt::{self, Formatter};

use crate::common::{BinaryOp, TempId, UnaryOp, VarName};

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
    pub body: Vec<CBlockItem>,
}

impl fmt::Display for CFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Function(name='{}', body={:?})", self.name, self.body,)
    }
}

#[derive(Debug, PartialEq)]
pub enum CStatement {
    Return(CExpression),
    Expression(CExpression),
    Null,
}

impl fmt::Display for CStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(exp) => {
                write!(f, "Return({})", exp)
            }
            Self::Expression(exp) => {
                write!(f, "Exp({})", exp)
            }
            Self::Null => {
                write!(f, "NULL")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CExpression {
    Factor(Box<CFactor>),
    Binary(BinaryOp, Box<CExpression>, Box<CExpression>),
    Assign(Box<CExpression>, Box<CExpression>), // TODO: replace left with CVar when doing resolution and parsing in one pass
}
impl fmt::Display for CExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factor(fac) => write!(f, "Factor({})", fac),
            Self::Binary(op, exp1, exp2) => write!(f, "BinaryOp({}, {}, {})", op, *exp1, *exp2),
            Self::Assign(exp1, exp2) => write!(f, "Assign({}, {})", exp1, exp2),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CFactor {
    Constant(i32),
    Unary(UnaryOp, Box<CFactor>),
    Expression(CExpression),
    Var(CVar),
}

impl fmt::Display for CFactor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(i) => write!(f, "Constant({})", i),
            Self::Unary(op, exp) => write!(f, "UnaryOp({}, {})", op, *exp),
            Self::Expression(exp) => write!(f, "Expression({})", exp),
            Self::Var(name) => write!(f, "Var({})", name),
        }
    }
}

// TODO: use named structs instead of tuples
#[derive(Debug, PartialEq, Clone)]
pub struct CDeclaration(pub CVar, pub Option<CExpression>);
impl CDeclaration {
    pub fn get_expression(self) -> Option<CExpression> {
        self.1
    }
    pub fn get_var(self) -> CVar {
        self.0
    }
}
impl fmt::Display for CDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.1 {
            Some(exp) => write!(f, "{}({})", self.0, exp),
            None => write!(f, "{}()", self.0),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CVar(pub VarName, pub Option<TempId>); // TODO: once variable resolution is moved to parsing pass, remove option
impl CVar {
    pub fn is_resolved(&self) -> bool {
        self.1.is_some()
    }
    pub fn get_name(&self) -> &VarName {
        &self.0
    }
}
impl fmt::Display for CVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.1 {
            Some(id) => write!(f, "{}.{}", &self.0, id.0),
            None => write!(f, "{}.?", &self.0),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CBlockItem {
    Statement(CStatement),
    Declaration(CDeclaration),
}
impl fmt::Display for CBlockItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Statement(stmnt) => write!(f, "{}", stmnt),
            Self::Declaration(decl) => write!(f, "{}", decl),
        }
    }
}
