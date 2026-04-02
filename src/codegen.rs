use crate::common::UnaryOp;
use crate::ir::*;
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
enum AsmInstruction {
    Mov {
        src: AsmOperand,
        dst: AsmOperand,
    },
    Unary {
        operator: AsmUnaryOp,
        operand: AsmOperand,
    },
    AllocateStack(usize),
    Ret,
}
impl fmt::Display for AsmInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mov { src, dst } => {
                write!(f, "movl {}, {}", src, dst)
            }
            Self::Unary { operator, operand } => write!(f, "{} {}", operator, operand),
            Self::AllocateStack(n_bytes) => write!(f, "subq ${}, %rsp", n_bytes),
            Self::Ret => write!(f, "movq %rbp, %rsp\npopq %rbp\nret"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum AsmOperand {
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
enum AsmUnaryOp {
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

#[derive(Debug, PartialEq, Clone)]
enum AsmRegister {
    Eax,
    R10d,
}
impl fmt::Display for AsmRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eax => write!(f, "eax"),
            Self::R10d => write!(f, "r10d"),
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

        for instruction in &mut asm_program.function.instructions {
            match instruction {
                AsmInstruction::Mov { src, dst } => {
                    self.pseudo_to_stack(src, &mut curr_offset, &mut tmp_to_offset);
                    self.pseudo_to_stack(dst, &mut curr_offset, &mut tmp_to_offset);
                }
                AsmInstruction::Unary {
                    operator: _,
                    operand,
                } => self.pseudo_to_stack(operand, &mut curr_offset, &mut tmp_to_offset),
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

    fn fix_instructions(&self, asm_program: &mut AsmProgram, stack_size: usize) {
        let mut fixed = vec![AsmInstruction::AllocateStack(stack_size)];

        for instruction in &mut asm_program.function.instructions.drain(..) {
            match instruction {
                AsmInstruction::Mov { src, dst }
                    if matches!(src, AsmOperand::Stack(_))
                        && matches!(dst, AsmOperand::Stack(_)) =>
                {
                    fixed.push(AsmInstruction::Mov {
                        src,
                        dst: AsmOperand::Register(AsmRegister::R10d),
                    });
                    fixed.push(AsmInstruction::Mov {
                        src: AsmOperand::Register(AsmRegister::R10d),
                        dst,
                    });
                }
                _ => fixed.push(instruction),
            }
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
            match ins {
                IRInstruction::Return(val) => {
                    instructions.push(AsmInstruction::Mov {
                        src: self.val_to_operand(val),
                        dst: AsmOperand::Register(AsmRegister::Eax),
                    });
                    instructions.push(AsmInstruction::Ret);
                }
                IRInstruction::Unary { op, src, dst } => {
                    let dst = self.val_to_operand(dst);
                    instructions.push(AsmInstruction::Mov {
                        src: self.val_to_operand(src),
                        dst: dst.clone(),
                    });
                    instructions.push(AsmInstruction::Unary {
                        operator: self.translate_unop(op),
                        operand: dst,
                    });
                }
                IRInstruction::Binary { op, src1, src2, dst } => todo!()
            }
        }
        instructions
    }

    fn val_to_operand(&self, val: IRVal) -> AsmOperand {
        match val {
            IRVal::Constant(i) => AsmOperand::Imm(i),
            IRVal::Var(tmp) => AsmOperand::PseudoReg(tmp),
        }
    }

    fn translate_unop(&self, op: UnaryOp) -> AsmUnaryOp {
        match op {
            UnaryOp::BitwiseComplement => AsmUnaryOp::Not,
            UnaryOp::Negation => AsmUnaryOp::Neg,
            _ => unimplemented!(),
        }
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

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::fs::File;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn return_2() {
        let code = b"int main(void) {
        return 2;
        }";

        let tokens = Lexer::new(code, "foo.c".to_string()).get_tokens();

        let program = Parser::new(tokens).parse_program().unwrap();
        let program = IRGenerator::new().c_to_ir(program);

        let codegen = AssemblyGenerator::new();

        let translated = codegen.ir_to_asm(program);

        let asm = codegen.generate_asm(translated);

        let tmp_dir = tempdir().expect("Failed to create temporary directory");
        let outpath = tmp_dir.path().join("return2.s");
        let mut file = File::create(&outpath).expect("Failed to create temporary file");

        write!(file, "{}", asm).expect("Failed to write file");

        let exe_path = tmp_dir.path().join("return2");
        let status = Command::new("gcc")
            .args([
                outpath.as_os_str().to_str().unwrap(),
                "-o",
                exe_path.as_os_str().to_str().unwrap(),
            ])
            .status()
            .expect("assembling/linking failed");

        assert!(status.success());

        let output = Command::new(exe_path).output().unwrap();

        assert_eq!(output.status.code().unwrap(), 2);
    }
    
    #[test]
    fn return_not_neg_2() {
        let code = b"int main(void) {
        return ~(-2);
        }";

        let tokens = Lexer::new(code, "foo.c".to_string()).get_tokens();

        let program = Parser::new(tokens).parse_program().unwrap();
        let program = IRGenerator::new().c_to_ir(program);

        let codegen = AssemblyGenerator::new();

        let translated = codegen.ir_to_asm(program);
        let asm = codegen.generate_asm(translated);

        let tmp_dir = tempdir().expect("Failed to create temporary directory");
        let outpath = tmp_dir.path().join("return2.s");
        let mut file = File::create(&outpath).expect("Failed to create temporary file");

        write!(file, "{}", asm).expect("Failed to write file");

        let exe_path = tmp_dir.path().join("return2");
        let status = Command::new("gcc")
            .args([
                outpath.as_os_str().to_str().unwrap(),
                "-o",
                exe_path.as_os_str().to_str().unwrap(),
            ])
            .status()
            .expect("assembling/linking failed");

        assert!(status.success());

        let output = Command::new(exe_path).output().unwrap();

        assert_eq!(output.status.code().unwrap(), 1);
    }
}
