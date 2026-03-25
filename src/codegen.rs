use crate::{ir::*, lexer::UnaryOp};

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
impl AsmInstruction {
    fn to_string(&self) -> String {
        match self {
            Self::Mov { src, dst } => {
                format!("movl {}, {}", src.to_string(), dst.to_string())
            }
            Self::Unary { operator, operand } => unimplemented!(),
            Self::AllocateStack(n_bytes) => format!("subq ${}, %rsp", n_bytes),
            Self::Ret => "ret".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum AsmOperand {
    Imm(i32),
    Register(AsmRegister),
    PseudoReg(TempId),
    Stack(i32),
}
impl AsmOperand {
    fn to_string(&self) -> String {
        match self {
            Self::Imm(i) => format!("${}", i),
            Self::Register(reg) => format!("%{}", reg.to_string()),
            Self::PseudoReg(tmp) => unimplemented!(),
            Self::Stack(offset) => format!("{}(%rbp)", offset),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AsmUnaryOp {
    Neg,
    Not,
}

#[derive(Debug, PartialEq, Clone)]
enum AsmRegister {
    EAX,
    R10D,
}
impl AsmRegister {
    fn to_string(&self) -> String {
        match self {
            Self::EAX => "EAX".to_string(),
            Self::R10D => unimplemented!(),
        }
    }
}

pub struct AssemblyGenerator {}
impl AssemblyGenerator {
    pub fn new() -> Self {
        AssemblyGenerator {}
    }

    pub fn ir_to_asm(&self, ir_program: IRProgram) -> AsmProgram {
        let mut asm_program = self.translate_program(ir_program);
        asm_program = self.replace_pseudo_registers(asm_program);
        self.fix_instructions(asm_program)
    }

    fn replace_pseudo_registers(&self, asm_program: AsmProgram) -> AsmProgram {
        todo!() // in-place instead of returning?
    }

    fn fix_instructions(&self, asm_program: AsmProgram) -> AsmProgram {
        todo!()
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
                        dst: AsmOperand::Register(AsmRegister::EAX),
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

        let tokens = Lexer::new(code).get_tokens();

        let program = Parser::new(tokens).parse_program().unwrap();
        let program = IRGenerator::new().c_to_ir(program);

        let codegen = AssemblyGenerator::new();

        let translated = codegen.ir_to_asm(program);

        let ref_translation = AsmProgram {
            function: AsmFunction {
                name: "main".to_string(),
                instructions: vec![
                    AsmInstruction::Mov {
                        src: AsmOperand::Imm(2),
                        dst: AsmOperand::Register(AsmRegister::EAX),
                    },
                    AsmInstruction::Ret,
                ],
            },
        };

        assert_eq!(translated, ref_translation);

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
}
