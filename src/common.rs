use std::fmt::{self, Formatter};

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Int,
    Void,
    Return,
}

impl Keyword {
    pub fn from_u8(s: &[u8]) -> Option<Self> {
        match s {
            b"int" => Some(Keyword::Int),
            b"void" => Some(Keyword::Void),
            b"return" => Some(Keyword::Return),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    BitwiseComplement,
    Negation,
    Decrement,
}
impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BitwiseComplement => write!(f, "~"),
            Self::Negation => write!(f, "-"),
            Self::Decrement => write!(f, "--"),
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
}
impl BinaryOp {
    pub fn precedence(&self) -> u32 {
        match self {
            Self::Add => 45,
            Self::Sub => 45,
            Self::Mul => 50,
            Self::Div => 50,
            Self::Mod => 50,
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
        }
    }
}