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
    If(CExpression, Box<CStatement>, Option<Box<CStatement>>), // condition, then, else
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
            Self::If(cond, then, else_) => match else_ {
                Some(stmnt) => write!(f, "If(cond={}, then={}, else={}", cond, then, stmnt),
                None => write!(f, "If(cond={}, then={}", cond, then),
            },
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
    Assign(Box<CExpression>, Box<CExpression>),
    Conditional(Box<CExpression>, Box<CExpression>, Box<CExpression>), // cond, then, else
}
impl fmt::Display for CExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factor(fac) => write!(f, "Factor({})", fac),
            Self::Binary(op, exp1, exp2) => write!(f, "BinaryOp({}, {}, {})", op, exp1, exp2),
            Self::Assign(exp1, exp2) => write!(f, "Assign({} = {})", exp1, exp2),
            Self::Conditional(cond, exp1, exp2) => {
                write!(f, "Conditional({} ? {} : {})", cond, exp1, exp2)
            }
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

#[derive(Debug, PartialEq, Clone)]
pub struct CDeclaration {
    pub var: CVar,
    pub init: Option<CExpression>,
}
impl fmt::Display for CDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.init {
            Some(exp) => write!(f, "{}({})", self.var, exp),
            None => write!(f, "{}()", self.var),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CVar {
    pub name: VarName,
    pub id: Option<TempId>,
}
impl fmt::Display for CVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "{}.{}", self.name, id.0),
            None => write!(f, "{}.?", self.name),
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
