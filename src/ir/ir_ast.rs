use std::fmt;

use crate::codegen::asm_ast::{
    AsmBinaryOp, AsmCondCode,
    AsmInstruction::{self, *},
    AsmOperand::{self, *},
    AsmRegister::{self, *},
    AsmUnaryOp,
};
use crate::common::{BinaryOp, TempId, UnaryOp};

#[derive(Debug, PartialEq)]
pub struct IRProgram {
    pub function: IRFunction,
}

#[derive(Debug, PartialEq)]
pub struct IRFunction {
    pub name: String,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum IRInstruction {
    Return(IRVal),
    Unary(UnaryOp, IRVal, IRVal),          // op, src, dst
    Binary(BinaryOp, IRVal, IRVal, IRVal), // op, src1, src2, dst
    Copy(IRVal, IRVal),
    Jump(Label),
    JumpIfZero(IRVal, Label),
    JumpIfNotZero(IRVal, Label),
    Label(Label),
}
impl IRInstruction {
    pub fn to_asm(&self) -> Vec<AsmInstruction> {
        match self {
            Self::Return(val) => {
                vec![Mov(val.to_asm(), Register(Eax)), Ret]
            }
            Self::Unary(op, src, dst) => match op {
                UnaryOp::LogicalNot => {
                    vec![
                        Cmp(Imm(0), src.to_asm()),
                        Mov(Imm(0), dst.to_asm()),
                        SetCC(AsmCondCode::E, dst.to_asm()),
                    ]
                }
                _ => {
                    vec![
                        Mov(src.to_asm(), dst.to_asm()),
                        Unary(translate_unop(op.clone()), dst.to_asm()),
                    ]
                }
            },
            Self::Binary(op, src1, src2, dst) => match op {
                BinaryOp::Div | BinaryOp::Mod => {
                    let src_reg = match op {
                        BinaryOp::Div => AsmRegister::Eax,
                        BinaryOp::Mod => AsmRegister::Edx,
                        _ => panic!(
                            "This arm should be unreachable because op has to be one of the above. 100% Rust bug not mine."
                        ),
                    };
                    vec![
                        Mov(src1.to_asm(), AsmOperand::Register(AsmRegister::Eax)),
                        Cdq,
                        Idiv(src2.to_asm()),
                        Mov(AsmOperand::Register(src_reg), dst.to_asm()),
                    ]
                }
                BinaryOp::Eq
                | BinaryOp::Neq
                | BinaryOp::Less
                | BinaryOp::Greater
                | BinaryOp::Leq
                | BinaryOp::Geq => {
                    let cond = match op {
                        BinaryOp::Eq => AsmCondCode::E,
                        BinaryOp::Neq => AsmCondCode::NE,
                        BinaryOp::Less => AsmCondCode::L,
                        BinaryOp::Greater => AsmCondCode::G,
                        BinaryOp::Leq => AsmCondCode::LE,
                        BinaryOp::Geq => AsmCondCode::GE,
                        _ => panic!("100% Rust bug not mine"),
                    };
                    vec![
                        Cmp(src2.to_asm(), src1.to_asm()),
                        Mov(AsmOperand::Imm(0), dst.to_asm()),
                        SetCC(cond, dst.to_asm()),
                    ]
                }
                _ => {
                    vec![
                        Mov(src1.to_asm(), dst.to_asm()),
                        Binary(translate_binop(op.clone()), src2.to_asm(), dst.to_asm()),
                    ]
                }
            },
            Self::Copy(src, dst) => {
                vec![Mov(src.to_asm(), dst.to_asm())]
            }
            Self::Jump(label) => vec![Jmp(*label)],
            Self::JumpIfZero(condition, target) => {
                vec![
                    Cmp(Imm(0), condition.to_asm()),
                    JmpCC(AsmCondCode::E, *target),
                ]
            }
            Self::JumpIfNotZero(condition, target) => {
                vec![
                    Cmp(Imm(0), condition.to_asm()),
                    JmpCC(AsmCondCode::NE, *target),
                ]
            }
            Self::Label(label) => vec![AsmInstruction::Label(*label)],
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IRVal {
    Constant(i32),
    Var(TempId),
}
impl IRVal {
    pub fn to_asm(&self) -> AsmOperand {
        match self {
            Self::Constant(i) => AsmOperand::Imm(*i),
            Self::Var(tmp) => AsmOperand::PseudoReg(*tmp),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Label {
    pub kind: LabelKind,
    pub id: u32,
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "label_{}_{}", self.kind, self.id)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LabelKind {
    True,
    False,
    End,
    LoopStart,
    Break,
    Continue,
    Else,
}
impl fmt::Display for LabelKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::End => write!(f, "end"),
            Self::LoopStart => write!(f, "start_loop"),
            Self::Break => write!(f, "break_loop"),
            Self::Continue => write!(f, "continue_loop"),
            Self::Else => write!(f, "else"),
        }
    }
}

fn translate_unop(op: UnaryOp) -> AsmUnaryOp {
    match op {
        UnaryOp::BitwiseNot => AsmUnaryOp::Not,
        UnaryOp::Negation => AsmUnaryOp::Neg,
        _ => unimplemented!(),
    }
}

fn translate_binop(op: BinaryOp) -> AsmBinaryOp {
    match op {
        BinaryOp::Add => AsmBinaryOp::Add,
        BinaryOp::Sub => AsmBinaryOp::Sub,
        BinaryOp::Mul => AsmBinaryOp::Imul,
        BinaryOp::Div | BinaryOp::Mod => unimplemented!(), // correspond to AsmInstruction::Idiv and are handled separately
        _ => todo!("logical ops"),
    }
}
