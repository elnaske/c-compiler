use std::fmt;

use crate::codegen::{
    AsmBinaryOp, AsmCondCode,
    AsmInstruction::{self, *},
    AsmOperand::{self, *},
    AsmRegister::{self, *},
    AsmUnaryOp,
};
use crate::common::{BinaryOp, UnaryOp};
use crate::parser::*;

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
pub struct TempId(u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Label(u32);
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "label_{}", self.0)
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

pub struct IRGenerator {
    next_var_id: u32,
    next_label_id: u32,
}
impl Default for IRGenerator {
    fn default() -> Self {
        Self::new()
    }
}
impl IRGenerator {
    pub fn new() -> Self {
        IRGenerator {
            next_var_id: 0,
            next_label_id: 0,
        }
    }

    fn create_temp_var(&mut self) -> TempId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TempId(id)
    }

    fn create_jump_label(&mut self) -> Label {
        let id = self.next_label_id;
        self.next_label_id += 1;
        Label(id)
    }

    pub fn c_to_ir(&mut self, c_program: CProgram) -> IRProgram {
        IRProgram {
            function: self.translate_function(c_program.function),
        }
    }

    fn translate_function(&mut self, c_function: CFunction) -> IRFunction {
        IRFunction {
            name: c_function.name,
            instructions: self.statement_to_instructions(c_function.body),
        }
    }

    fn statement_to_instructions(&mut self, c_statement: CStatement) -> Vec<IRInstruction> {
        let mut instructions = Vec::<IRInstruction>::new();

        match c_statement {
            CStatement::Return(exp) => {
                let (return_val, mut exp_instructions) = self.exp_to_instructions(exp);
                instructions.append(&mut exp_instructions);
                instructions.push(IRInstruction::Return(return_val));
            }
        }
        instructions
    }

    fn exp_to_instructions(&mut self, c_expression: CExpression) -> (IRVal, Vec<IRInstruction>) {
        match c_expression {
            CExpression::Factor(f) => self.factor_to_instructions(*f),
            CExpression::Binary(op, exp1, exp2) => self.binop_to_instructions(op, *exp1, *exp2),
        }
    }

    fn factor_to_instructions(&mut self, c_factor: CFactor) -> (IRVal, Vec<IRInstruction>) {
        match c_factor {
            CFactor::Constant(i) => (IRVal::Constant(i), vec![]),
            CFactor::Unary(op, inner_exp) => {
                let (src, mut inner_instructions) = self.factor_to_instructions(*inner_exp);
                let dst = IRVal::Var(self.create_temp_var());

                inner_instructions.push(IRInstruction::Unary(op, src, dst));
                
                (dst, inner_instructions)
            }
            CFactor::Expression(exp) => self.exp_to_instructions(*exp),
        }
    }

    fn binop_to_instructions(
        &mut self,
        op: BinaryOp,
        exp1: CExpression,
        exp2: CExpression,
    ) -> (IRVal, Vec<IRInstruction>) {
        use IRInstruction::*;

        let (src1, ins1) = self.exp_to_instructions(exp1);
        let (src2, ins2) = self.exp_to_instructions(exp2);
        let dst = IRVal::Var(self.create_temp_var());

        let instructions = match op {
            BinaryOp::LogicalAnd => {
                let false_label = self.create_jump_label();
                let end_label = self.create_jump_label();

                vec![
                    ins1,
                    vec![JumpIfZero(src1, false_label)],
                    ins2,
                    vec![
                        JumpIfZero(src2, false_label),
                        Copy(IRVal::Constant(1), dst),
                        Jump(end_label),
                        Label(false_label),
                        Copy(IRVal::Constant(0), dst),
                        Label(end_label),
                    ],
                ]
            }
            BinaryOp::LogicalOr => {
                let true_label = self.create_jump_label();
                let end_label = self.create_jump_label();

                vec![
                    ins1,
                    vec![JumpIfNotZero(src1, true_label)],
                    ins2,
                    vec![
                        JumpIfNotZero(src2, true_label),
                        Copy(IRVal::Constant(0), dst),
                        Jump(end_label),
                        Label(true_label),
                        Copy(IRVal::Constant(1), dst),
                        Label(end_label),
                    ],
                ]
            }
            _ => {
                vec![ins1, ins2, vec![Binary(op, src1, src2, dst)]]
            }
        }
        .into_iter()
        .flatten()
        .collect();

        (dst, instructions)
    }
}
