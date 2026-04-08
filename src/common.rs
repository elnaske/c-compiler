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
pub enum Operator {
    BitwiseNot,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Decrement,
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
            _ => None
        }
    }
    pub fn to_binop(&self) -> Option<BinaryOp> {
        match self {
            Self::Plus => Some(BinaryOp::Add),
            Self::Minus => Some(BinaryOp::Sub),
            Self::Mul => Some(BinaryOp::Mul),
            Self::Div => Some(BinaryOp::Div),
            Self::Mod => Some(BinaryOp::Mod),
            _ => None
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
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    BitwiseNot,
    Negation,
    Decrement,
}
impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BitwiseNot => write!(f, "~"),
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