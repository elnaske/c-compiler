use std::fmt::{self, Formatter};
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Int,
    Void,
    If,
    Else,
    Return,
}

impl Keyword {
    pub fn from_u8(s: &[u8]) -> Option<Self> {
        match s {
            b"int" => Some(Keyword::Int),
            b"void" => Some(Keyword::Void),
            b"if" => Some(Keyword::If),
            b"else" => Some(Keyword::Else),
            b"return" => Some(Keyword::Return),
            _ => None,
        }
    }
}

// TODO: this is too much repeated code, go back to the way it was before (or find a better way)
// TODO: try implementing copy trait to avoid using clone() everywhere
#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    BitwiseNot,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Decrement,
    LogicalNot,
    LogicalAnd,
    LogicalOr,
    Eq,
    Neq,
    Less,
    Greater,
    Leq,
    Geq,
    Assign,
    Conditional,
}
impl Operator {
    pub fn is_unary(&self) -> bool {
        self.to_unop().is_some()
    }
    pub fn is_binary(&self) -> bool {
        self.to_binop().is_some()
    }
    pub fn precedence(&self) -> u32 {
        match self.to_binop() {
            Some(binop) => binop.precedence(),
            None => 0,
        }
    }
    pub fn to_unop(&self) -> Option<UnaryOp> {
        match self {
            Self::BitwiseNot => Some(UnaryOp::BitwiseNot),
            Self::Minus => Some(UnaryOp::Negation),
            Self::Decrement => Some(UnaryOp::Decrement),
            Self::LogicalNot => Some(UnaryOp::LogicalNot),
            _ => None,
        }
    }
    pub fn to_binop(&self) -> Option<BinaryOp> {
        match self {
            Self::Plus => Some(BinaryOp::Add),
            Self::Minus => Some(BinaryOp::Sub),
            Self::Mul => Some(BinaryOp::Mul),
            Self::Div => Some(BinaryOp::Div),
            Self::Mod => Some(BinaryOp::Mod),
            Self::LogicalAnd => Some(BinaryOp::LogicalAnd),
            Self::LogicalOr => Some(BinaryOp::LogicalOr),
            Self::Eq => Some(BinaryOp::Eq),
            Self::Neq => Some(BinaryOp::Neq),
            Self::Less => Some(BinaryOp::Less),
            Self::Greater => Some(BinaryOp::Greater),
            Self::Leq => Some(BinaryOp::Leq),
            Self::Geq => Some(BinaryOp::Geq),
            Self::Assign => Some(BinaryOp::Assign),
            Self::Conditional => Some(BinaryOp::Conditional),
            _ => None,
        }
    }
}
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BitwiseNot => write!(f, "~"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Decrement => write!(f, "--"),
            Self::LogicalNot => write!(f, "!"),
            Self::LogicalAnd => write!(f, "&&"),
            Self::LogicalOr => write!(f, "||"),
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::Leq => write!(f, "<="),
            Self::Geq => write!(f, ">="),
            Self::Assign => write!(f, "="),
            Self::Conditional => write!(f, "?"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    BitwiseNot,
    Negation,
    Decrement,
    LogicalNot,
}
impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BitwiseNot => write!(f, "~"),
            Self::Negation => write!(f, "-"),
            Self::Decrement => write!(f, "--"),
            Self::LogicalNot => write!(f, "!"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogicalAnd,
    LogicalOr,
    Eq,
    Neq,
    Less,
    Greater,
    Leq,
    Geq,
    Assign,
    Conditional, // ternary, but who cares
}
impl BinaryOp {
    pub fn precedence(&self) -> u32 {
        match self {
            Self::Mul => 50,
            Self::Div => 50,
            Self::Mod => 50,
            Self::Add => 45,
            Self::Sub => 45,
            Self::Less => 35,
            Self::Leq => 35,
            Self::Greater => 35,
            Self::Geq => 35,
            Self::Eq => 30,
            Self::Neq => 30,
            Self::LogicalAnd => 10,
            Self::LogicalOr => 5,
            Self::Conditional => 3,
            Self::Assign => 1,
        }
    }
}
impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::LogicalAnd => write!(f, "&&"),
            Self::LogicalOr => write!(f, "||"),
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::Leq => write!(f, "<="),
            Self::Geq => write!(f, ">="),
            Self::Assign => write!(f, "="),
            Self::Conditional => write!(f, "?"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TempId(pub u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct VarName(pub String);
impl Deref for VarName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl fmt::Display for VarName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
impl From<String> for VarName {
    fn from(s: String) -> Self {
        VarName(s)
    }
}
