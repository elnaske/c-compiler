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
    pub functions: Vec<IRFunction>,
}

#[derive(Debug, PartialEq)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<String>,
    pub param_ids: Vec<TempId>,
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
    FnCall(String, Vec<IRVal>, IRVal), // name, args, dst
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
            Self::FnCall(name, args, dst) => {
                let mut instructions = Vec::<AsmInstruction>::new();
                let arg_registers = [
                    AsmRegister::Edi,
                    AsmRegister::Esi,
                    AsmRegister::Edx,
                    AsmRegister::Ecx,
                    AsmRegister::R8d,
                    AsmRegister::R9d,
                ];

                let mut register_args = Vec::<IRVal>::new();
                let mut stack_args = Vec::<IRVal>::new();
                let mut args = args.clone();

                // TODO: check if the if is necessary (can range for drain() be bigger than len?)
                if args.len() > arg_registers.len() {
                    register_args.append(&mut args.drain(..6).collect());
                    stack_args.append(&mut args.drain(..).collect());
                } else {
                    register_args = args;
                    stack_args = vec![];
                }

                // align to 16 bytes
                let stack_padding: usize = { if stack_args.len() % 2 == 1 { 8 } else { 0 } };
                if stack_padding > 0 {
                    instructions.push(AllocateStack(stack_padding))
                }

                // pass args in registers
                for (i, arg) in register_args.iter().enumerate() {
                    instructions.push(Mov(arg.to_asm(), Register(arg_registers[i])));
                }

                // pass args on stack in reverse order
                for arg in stack_args.iter().rev() {
                    let asm_arg = arg.to_asm();
                    match asm_arg {
                        Imm(_) | Register(_) => {
                            instructions.push(Push(asm_arg));
                        }
                        PseudoReg(_) | Stack(_) => instructions
                            .append(&mut vec![Mov(asm_arg, Register(Eax)), Push(Register(Eax))]),
                    }
                }

                instructions.push(Call(name.clone()));

                let bytes_to_remove = 8 * stack_args.len() + stack_padding;
                if bytes_to_remove > 0 {
                    instructions.push(DellocateStack(bytes_to_remove))
                }

                // get return value
                instructions.push(Mov(Register(Eax), dst.to_asm()));

                instructions
            }
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
