use std::fmt::{self, Formatter};
use std::ops::Deref;

use crate::common::{BinaryOp, Keyword, TempId, UnaryOp};
use crate::ir::ir_ast::Label;

#[derive(Debug, PartialEq)]
pub struct CProgram {
    pub functions: Vec<CFnDecl>,
}
impl fmt::Display for CProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Program({:?})", self.functions)
    }
}

#[derive(Debug, PartialEq)]
pub struct CFnDecl {
    pub name: String,
    pub params: Vec<CParam>,
    pub body: Option<CBlock>,
}
impl fmt::Display for CFnDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Function(name='{}', body={:?})", self.name, self.body,)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CParam {
    pub keyword: Keyword,
    pub name: Option<String>, // only None for void
    pub id: Option<TempId>,
}

#[derive(Debug, PartialEq)]
pub struct CBlock(pub Vec<CBlockItem>);
impl Deref for CBlock {
    type Target = Vec<CBlockItem>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl fmt::Display for CBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Block({:?})", &self.0)
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

#[derive(Debug, PartialEq)]
pub enum CForInit {
    InitDecl(CVarDecl),
    InitExp(Option<CExpression>),
}

#[derive(Debug, PartialEq)]
pub enum CStatement {
    Return(CExpression),
    Expression(CExpression),
    If {
        cond: CExpression,
        then: Box<CStatement>,
        else_: Option<Box<CStatement>>,
    },
    Compound(CBlock),
    Break(Option<Label>),
    Continue(Option<Label>),
    While {
        cond: CExpression,
        body: Box<CStatement>,
        label: Option<Label>,
    },
    DoWhile {
        body: Box<CStatement>,
        cond: CExpression,
        label: Option<Label>,
    },
    For {
        init: CForInit,
        cond: Option<CExpression>,
        post: Option<CExpression>,
        body: Box<CStatement>,
        label: Option<Label>,
    },
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
            Self::If { cond, then, else_ } => match else_ {
                Some(stmnt) => write!(f, "If(cond={}, then={}, else={}", cond, then, stmnt),
                None => write!(f, "If(cond={}, then={}", cond, then),
            },
            Self::Compound(block) => write!(f, "Block({:?})", block),
            Self::Break(_label) => write!(f, "Break"),
            Self::Continue(_label) => write!(f, "Continue"),
            Self::While {
                cond,
                body,
                label: _,
            } => write!(f, "While(cond={}, do={})", cond, *body),
            Self::DoWhile {
                body,
                cond,
                label: _,
            } => write!(f, "DoWhile(do={}, cond={})", *body, cond),
            Self::For {
                init,
                cond,
                post,
                body,
                label: _,
            } => {
                write!(
                    f,
                    "For(init={:?}, cond={:?}, post={:?}, do={:?})",
                    init, cond, post, body
                )
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
    Assign(Box<CExpression>, Box<CExpression>),
    Conditional {
        cond: Box<CExpression>,
        then: Box<CExpression>,
        else_: Box<CExpression>,
    },
}
impl fmt::Display for CExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factor(fac) => write!(f, "Factor({})", fac),
            Self::Binary(op, exp1, exp2) => write!(f, "BinaryOp({}, {}, {})", op, exp1, exp2),
            Self::Assign(exp1, exp2) => write!(f, "Assign({} = {})", exp1, exp2),
            Self::Conditional { cond, then, else_ } => {
                write!(f, "Conditional({} ? {} : {})", cond, then, else_)
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
    FunctionCall(String, Vec<CExpression>), // name, args
}
impl fmt::Display for CFactor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(i) => write!(f, "Constant({})", i),
            Self::Unary(op, exp) => write!(f, "UnaryOp({}, {})", op, *exp),
            Self::Expression(exp) => write!(f, "Expression({})", exp),
            Self::Var(name) => write!(f, "Var({})", name),
            Self::FunctionCall(name, args) => {
                write!(f, "{}({:?})", name, args)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CDeclaration {
    FnDecl(CFnDecl),
    VarDecl(CVarDecl),
}
impl fmt::Display for CDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FnDecl(dec) => write!(f, "{}", dec),
            Self::VarDecl(dec) => write!(f, "{}", dec),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CVarDecl {
    pub var: CVar,
    pub init: Option<CExpression>,
}
impl fmt::Display for CVarDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.init {
            Some(exp) => write!(f, "{}({})", self.var, exp),
            None => write!(f, "{}()", self.var),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CVar {
    pub name: String,
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
