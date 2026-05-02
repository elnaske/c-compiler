use crate::common::TempId;
use crate::ir::ir_ast::Label;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct AsmProgram {
    pub functions: Vec<AsmFunction>,
}

#[derive(Debug, PartialEq)]
pub struct AsmFunction {
    pub name: String,
    pub instructions: Vec<AsmInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum AsmInstruction {
    Mov(AsmOperand, AsmOperand),
    Unary(AsmUnaryOp, AsmOperand),
    Binary(AsmBinaryOp, AsmOperand, AsmOperand),
    Cmp(AsmOperand, AsmOperand),
    Idiv(AsmOperand),
    Cdq,
    Jmp(Label),
    JmpCC(AsmCondCode, Label),
    SetCC(AsmCondCode, AsmOperand),
    Label(Label),
    AllocateStack(usize),
    DeallocateStack(usize),
    Push(AsmOperand),
    Call(String),
    Ret,
}
impl AsmInstruction {
    pub fn fix(self) -> Vec<AsmInstruction> {
        use AsmOperand::*;
        use AsmRegister::*;

        match self {
            Self::Mov(src, dst)
                if matches!(src, AsmOperand::Stack(_)) && matches!(dst, AsmOperand::Stack(_)) =>
            {
                // src and dst for mov can't both be memory addresses, use %r10d as an intermediate step
                vec![
                    Self::Mov(src, Register(R10d)),
                    Self::Mov(Register(R10d), dst),
                ]
            }
            Self::Binary(operator, operand1, operand2) => {
                match operator {
                    AsmBinaryOp::Add | AsmBinaryOp::Sub
                        if matches!(operand1, Stack(_)) && matches!(operand2, Stack(_)) =>
                    {
                        // same as with mov, both operands can't be memory addresses
                        vec![
                            Self::Mov(operand1, Register(R10d)),
                            Self::Binary(operator, Register(R10d), operand2),
                        ]
                    }
                    AsmBinaryOp::Imul if matches!(operand2, Stack(_)) => {
                        // destination can't be memory address
                        vec![
                            Self::Mov(operand2.clone(), Register(R11d)),
                            Self::Binary(operator, operand1, Register(R11d)),
                            Self::Mov(Register(R11d), operand2),
                        ]
                    }
                    _ => vec![Self::Binary(operator, operand1, operand2)],
                }
            }
            Self::Cmp(op1, op2) if matches!(op1, Stack(_)) && matches!(op2, Stack(_)) => {
                vec![
                    Self::Mov(op1, Register(R10d)),
                    Self::Cmp(Register(R10d), op2),
                ]
            }
            Self::Cmp(op1, op2) if matches!(op2, Imm(_)) => {
                vec![
                    Self::Mov(op2, Register(R11d)),
                    Self::Cmp(op1, Register(R11d)),
                ]
            }
            Self::Idiv(op) if matches!(op, Imm(_)) => {
                // operand can't be an immediate value
                vec![Self::Mov(op, Register(R10d)), Self::Idiv(Register(R10d))]
            }
            _ => vec![self],
        }
    }
}
impl fmt::Display for AsmInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mov(src, dst) => {
                write!(f, "movl {}, {}", src, dst)
            }
            Self::Unary(operator, operand) => write!(f, "{} {}", operator, operand),
            Self::Binary(operator, operand1, operand2) => {
                write!(f, "{} {}, {}", operator, operand1, operand2)
            }
            Self::Cmp(op1, op2) => write!(f, "cmpl {}, {}", op1, op2),
            Self::Idiv(op) => write!(f, "idivl {}", op),
            Self::Cdq => write!(f, "cdq"),
            Self::Jmp(target) => write!(f, "jmp .L_{}", target),
            Self::JmpCC(cond, target) => write!(f, "j{} .L_{}", cond, target),
            Self::SetCC(cond, op) => match op {
                AsmOperand::Register(reg) => write!(f, "set{} %{}", cond, reg.as_1_byte()),
                _ => write!(f, "set{} {}", cond, op),
            },
            Self::Label(label) => write!(f, ".L_{}:", label),
            Self::AllocateStack(n_bytes) => write!(f, "subq ${}, %rsp", n_bytes),
            Self::DeallocateStack(n_bytes) => write!(f, "addq ${}, %rsp", n_bytes),
            Self::Push(op) => match op {
                AsmOperand::Register(reg) => write!(f, "pushq %{}", reg.as_8_bytes()),
                _ => write!(f, "pushq {}", op),
            },
            Self::Call(fn_name) => {
                let mut fn_name = fn_name.clone();
                let _ = fn_name.split_off(fn_name.len() - 2);
                write!(f, "call {}", fn_name)
            } // external libraries (@PLT) handled in mod.rs
            Self::Ret => write!(f, "movq %rbp, %rsp\npopq %rbp\nret"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AsmOperand {
    Imm(i32),
    Register(AsmRegister),
    PseudoReg(TempId),
    Stack(i32),
}
impl fmt::Display for AsmOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Imm(i) => write!(f, "${}", i),
            Self::Register(reg) => write!(f, "%{}", reg),
            Self::PseudoReg(_tmp) => unimplemented!(),
            Self::Stack(offset) => write!(f, "{}(%rbp)", offset),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AsmUnaryOp {
    Neg,
    Not,
}
impl fmt::Display for AsmUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Neg => write!(f, "negl"),
            Self::Not => write!(f, "notl"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AsmBinaryOp {
    Add,
    Sub,
    Imul,
}
impl fmt::Display for AsmBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "addl"),
            Self::Sub => write!(f, "subl"),
            Self::Imul => write!(f, "imull"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AsmRegister {
    Eax,
    Ecx,
    Edx,
    Edi,
    Esi,
    R8d,
    R9d,
    R10d,
    R11d,
}
impl AsmRegister {
    fn as_1_byte(&self) -> String {
        match self {
            Self::Eax => "al",
            Self::Ecx => "cl",
            Self::Edx => "dl",
            Self::Edi => "dil",
            Self::Esi => "sil",
            Self::R8d => "r8b",
            Self::R9d => "r9b",
            Self::R10d => "r10b",
            Self::R11d => "r11b",
        }
        .to_string()
    }
    fn as_8_bytes(&self) -> String {
        match self {
            Self::Eax => "rax",
            Self::Ecx => "rcx",
            Self::Edx => "rdx",
            Self::Edi => "rdi",
            Self::Esi => "rsi",
            Self::R8d => "r8",
            Self::R9d => "r9",
            Self::R10d => "r10",
            Self::R11d => "r11",
        }
        .to_string()
    }
}
impl fmt::Display for AsmRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eax => write!(f, "eax"),
            Self::Ecx => write!(f, "ecx"),
            Self::Edx => write!(f, "edx"),
            Self::Edi => write!(f, "edi"),
            Self::Esi => write!(f, "esi"),
            Self::R8d => write!(f, "r8d"),
            Self::R9d => write!(f, "r9d"),
            Self::R10d => write!(f, "r10d"),
            Self::R11d => write!(f, "r11d"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AsmCondCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
}
impl fmt::Display for AsmCondCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::E => write!(f, "e"),
            Self::NE => write!(f, "ne"),
            Self::L => write!(f, "l"),
            Self::LE => write!(f, "le"),
            Self::G => write!(f, "g"),
            Self::GE => write!(f, "ge"),
        }
    }
}
