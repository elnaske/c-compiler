use crate::ir::*;
use std::vec;
use std::{collections::HashMap, fmt};

#[derive(Debug, PartialEq)]
pub struct AsmProgram {
    function: AsmFunction,
}

#[derive(Debug, PartialEq)]
struct AsmFunction {
    name: String,
    instructions: Vec<AsmInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum AsmInstruction {
    // TODO: use tuples here instead of structs
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
    Ret,
}
impl AsmInstruction {
    fn fix(self) -> Vec<AsmInstruction> {
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
            Self::Jmp(target) => write!(f, "jmp .L{}", target),
            Self::JmpCC(cond, target) => write!(f, "j{} .L{}", cond, target),
            Self::SetCC(cond, op) => match op {
                AsmOperand::Register(reg) => write!(f, "set{} {}", cond, reg.as_1_byte()),
                _ => write!(f, "set{} {}", cond, op),
            },
            Self::Label(label) => write!(f, ".L{}:", label),
            Self::AllocateStack(n_bytes) => write!(f, "subq ${}, %rsp", n_bytes),
            Self::Ret => write!(f, "movq %rbp, %rsp\npopq %rbp\nret"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AsmOperand {
    Imm(i32),
    Register(AsmRegister),
    PseudoReg(TempId),
    Stack(usize),
}
impl fmt::Display for AsmOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Imm(i) => write!(f, "${}", i),
            Self::Register(reg) => write!(f, "%{}", reg),
            Self::PseudoReg(_tmp) => unimplemented!(),
            Self::Stack(offset) => write!(f, "-{}(%rbp)", offset),
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

#[derive(Debug, PartialEq, Clone)]
pub enum AsmRegister {
    Eax,
    Edx,
    R10d,
    R11d,
}
impl AsmRegister {
    fn as_1_byte(&self) -> String {
        match self {
            Self::Eax => "al",
            Self::Edx => "dl",
            Self::R10d => "r10b",
            Self::R11d => "r11b",
        }
        .to_string()
    }
}
impl fmt::Display for AsmRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eax => write!(f, "eax"),
            Self::Edx => write!(f, "edx"),
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

pub struct AssemblyGenerator {}
impl Default for AssemblyGenerator {
    fn default() -> Self {
        Self::new()
    }
}
impl AssemblyGenerator {
    pub fn new() -> Self {
        AssemblyGenerator {}
    }

    pub fn ir_to_asm(&self, ir_program: IRProgram) -> AsmProgram {
        let mut asm_program = self.translate_program(ir_program);
        let stack_size = self.replace_pseudo_registers(&mut asm_program);
        self.fix_instructions(&mut asm_program, stack_size);
        asm_program
    }

    fn replace_pseudo_registers(&self, asm_program: &mut AsmProgram) -> usize {
        let mut tmp_to_offset = HashMap::<TempId, usize>::new();
        let mut curr_offset: usize = 0;

        // TODO: is there a better way to do this?
        for instruction in &mut asm_program.function.instructions {
            match instruction {
                AsmInstruction::Mov(src, dst) => {
                    self.pseudo_to_stack(src, &mut curr_offset, &mut tmp_to_offset);
                    self.pseudo_to_stack(dst, &mut curr_offset, &mut tmp_to_offset);
                }
                AsmInstruction::Unary(_, operand) => {
                    self.pseudo_to_stack(operand, &mut curr_offset, &mut tmp_to_offset)
                }
                AsmInstruction::Binary(_, operand1, operand2) => {
                    self.pseudo_to_stack(operand1, &mut curr_offset, &mut tmp_to_offset);
                    self.pseudo_to_stack(operand2, &mut curr_offset, &mut tmp_to_offset);
                }
                AsmInstruction::Cmp(op1, op2) => {
                    self.pseudo_to_stack(op1, &mut curr_offset, &mut tmp_to_offset);
                    self.pseudo_to_stack(op2, &mut curr_offset, &mut tmp_to_offset);
                }
                AsmInstruction::Idiv(operand) => {
                    self.pseudo_to_stack(operand, &mut curr_offset, &mut tmp_to_offset)
                }
                AsmInstruction::SetCC(_, op) => {
                    self.pseudo_to_stack(op, &mut curr_offset, &mut tmp_to_offset);
                }
                _ => (),
            }
        }
        curr_offset
    }

    fn pseudo_to_stack(
        &self,
        operand: &mut AsmOperand,
        curr_offset: &mut usize,
        tmp_to_offset: &mut HashMap<TempId, usize>,
    ) {
        if let AsmOperand::PseudoReg(tmp) = operand {
            let stack_offset = match tmp_to_offset.get(tmp) {
                Some(offset) => *offset,
                None => {
                    *curr_offset += 4;
                    tmp_to_offset.insert(tmp.clone(), *curr_offset);
                    *curr_offset
                }
            };
            *operand = AsmOperand::Stack(stack_offset);
        };
    }

    // TODO: try using an iterator here
    fn fix_instructions(&self, asm_program: &mut AsmProgram, stack_size: usize) {
        let mut fixed = vec![AsmInstruction::AllocateStack(stack_size)];

        for instruction in &mut asm_program.function.instructions.drain(..) {
            fixed.append(&mut instruction.fix());
            // // match instruction {
            // //     AsmInstruction::Mov { src, dst }
            // //         if matches!(src, AsmOperand::Stack(_))
            // //             && matches!(dst, AsmOperand::Stack(_)) =>
            // //     {
            // //         // src and dst for mov can't both be memory addresses, use %r10d as an intermediate step
            // //         vec![
            // //         Self::Mov {
            // //             src,
            // //             dst: AsmOperand::Register(AsmRegister::R10d),
            // //         },
            // //         Self::Mov {
            // //             src: AsmOperand::Register(AsmRegister::R10d),
            // //             dst,
            // //         }
            // //         ]
            // //     }
            // //     AsmInstruction::Binary {
            // //         operator,
            // //         operand1,
            // //         operand2,
            // //     } => {
            // //         match operator {
            // //             AsmBinaryOp::Add | AsmBinaryOp::Sub
            // //                 if matches!(operand1, AsmOperand::Stack(_))
            // //                     && matches!(operand2, AsmOperand::Stack(_)) =>
            // //             {
            // //                 // same as with mov, both operands can't be memory addresses
            // //                 Self::Mov {
            // //                     src: operand1,
            // //                     dst: AsmOperand::Register(AsmRegister::R10d),
            // //                 });
            // //                 Self::Binary {
            // //                     operator,
            // //                     operand1: AsmOperand::Register(AsmRegister::R10d),
            // //                     operand2,
            // //                 });
            // //             }
            // //             AsmBinaryOp::Imul if matches!(operand2, AsmOperand::Stack(_)) => {
            // //                 // destination can't be memory address
            // //                 Self::Mov {
            // //                     src: operand2.clone(),
            // //                     dst: AsmOperand::Register(AsmRegister::R11d),
            // //                 });
            // //                 Self::Binary {
            // //                     operator,
            // //                     operand1,
            // //                     operand2: AsmOperand::Register(AsmRegister::R11d),
            // //                 });
            // //                 Self::Mov {
            // //                     src: AsmOperand::Register(AsmRegister::R11d),
            // //                     dst: operand2,
            // //                 });
            // //             }
            // //             _ => Self::Binary {
            // //                 operator,
            // //                 operand1,
            // //                 operand2,
            // //             }),
            // //         }
            // //     }
            //     AsmInstruction::Cmp(op1, op2)
            //         if matches!(op1, AsmOperand::Stack(_))
            //             && matches!(op2, AsmOperand::Stack(_)) =>
            //     {
            //         Self::Mov {
            //             src: op1,
            //             dst: AsmOperand::Register(AsmRegister::R10d),
            //         });
            //         Self::Cmp(
            //             AsmOperand::Register(AsmRegister::R10d),
            //             op2,
            //         ));
            //     }
            //     AsmInstruction::Cmp(op1, op2) if matches!(op2, AsmOperand::Imm(_)) => {
            //         Self::Mov {
            //             src: op2,
            //             dst: AsmOperand::Register(AsmRegister::R11d),
            //         });
            //         Self::Cmp(
            //             op1,
            //             AsmOperand::Register(AsmRegister::R11d),
            //         ));
            //     }
            //     AsmInstruction::Idiv(op) if matches!(op, AsmOperand::Imm(_)) => {
            //         // operand can't be an immediate value
            //         Self::Mov {
            //             src: op,
            //             dst: AsmOperand::Register(AsmRegister::R10d),
            //         });
            //         Self::Idiv(AsmOperand::Register(
            //             AsmRegister::R10d,
            //         )));
            //     }
            //     _ => fixed.push(instruction),
            // }
        }
        asm_program.function.instructions = fixed;
    }

    pub fn translate_program(&self, ir_program: IRProgram) -> AsmProgram {
        AsmProgram {
            function: self.translate_function(ir_program.function),
        }
    }

    fn translate_function(&self, ir_function: IRFunction) -> AsmFunction {
        AsmFunction {
            name: ir_function.name,
            instructions: self.translate_instructions(ir_function.instructions),
        }
    }

    fn translate_instructions(&self, ir_instructions: Vec<IRInstruction>) -> Vec<AsmInstruction> {
        let mut instructions: Vec<AsmInstruction> = Vec::new();

        for ins in ir_instructions {
            instructions.append(&mut ins.to_asm());
        }
        instructions
    }

    pub fn generate_asm(&self, program: AsmProgram) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("\t.globl {}", program.function.name));
        lines.push(format!("{}:", program.function.name));
        lines.push("pushq %rbp".to_string());
        lines.push("movq %rsp, %rbp".to_string());
        lines.append(
            &mut program
                .function
                .instructions
                .into_iter()
                .map(|instr| instr.to_string())
                .collect::<Vec<String>>(),
        );
        lines.push("\t.section .note.GNU-stack\n".to_string());

        lines.join("\n")
    }
}
