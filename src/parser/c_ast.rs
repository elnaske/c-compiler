use std::fmt::{self, Formatter};

use crate::common::{BinaryOp, UnaryOp};

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
    pub body: CStatement,
}

impl fmt::Display for CFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Function(name='{}', body={})", self.name, self.body,)
    }
}

#[derive(Debug, PartialEq)]
pub enum CStatement {
    Return(CExpression),
}

impl fmt::Display for CStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(exp) => {
                write!(f, "Return({})", exp)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CExpression {
    Factor(Box<CFactor>),
    Binary(BinaryOp, Box<CExpression>, Box<CExpression>),
}
impl fmt::Display for CExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factor(fac) => write!(f, "Factor({})", fac),
            Self::Binary(op, exp1, exp2) => write!(f, "BinaryOp({}, {}, {})", op, *exp1, *exp2),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CFactor {
    Constant(i32),
    Unary(UnaryOp, Box<CFactor>),
    Expression(Box<CExpression>),
}

impl fmt::Display for CFactor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(i) => write!(f, "Constant({})", i),
            Self::Unary(op, exp) => write!(f, "UnaryOp({}, {})", op, *exp),
            Self::Expression(exp) => write!(f, "Expression({})", *exp),
        }
    }
}
