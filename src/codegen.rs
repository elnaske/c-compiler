use std::vec;

use crate::parser::*;

// TODO: separate mod or prefix for namespace

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
    Mov { src: AsmOperand, dst: AsmOperand },
    Ret,
}
impl AsmInstruction {
    fn to_string(&self) -> String {
        match self {
            AsmInstruction::Mov { src, dst } => {
                format!("\tmovl\t{}, {}", src.to_string(), dst.to_string())
            }
            AsmInstruction::Ret => "\tret".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AsmOperand {
    Imm(i32),
    Register(AsmRegister),
}
impl AsmOperand {
    fn to_string(&self) -> String {
        match self {
            AsmOperand::Imm(i) => format!("${}", i),
            AsmOperand::Register(reg) => format!("%{}", reg.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AsmRegister {
    EAX,
}
impl AsmRegister {
    fn to_string(&self) -> String {
        match self {
            AsmRegister::EAX => "EAX".to_string(),
        }
    }
}

// TODO: refactor to use IR
// TODO: remove AssemblyGenerator and use impl on types (?)
pub struct AssemblyGenerator {}
impl AssemblyGenerator {
    pub fn new() -> Self {
        AssemblyGenerator {}
    }

    pub fn translate(&self, c_program: CProgram) -> AsmProgram {
        AsmProgram {
            function: self.translate_function(c_program.function),
        }
    }

    fn translate_function(&self, c_function: CFunction) -> AsmFunction {
        AsmFunction {
            name: c_function.name,
            instructions: self.translate_statement(c_function.body),
        }
    }

    fn translate_statement(&self, statement: CStatement) -> Vec<AsmInstruction> {
        let mut instructions: Vec<AsmInstruction> = Vec::new();

        match statement {
            CStatement::Return(exp) => {
                instructions.append(&mut self.translate_expression(exp));
                instructions.push(AsmInstruction::Ret);
            }
        }
        instructions
    }

    fn translate_expression(&self, expression: CExpression) -> Vec<AsmInstruction> {
        match expression {
            CExpression::Constant(i) => vec![AsmInstruction::Mov {
                src: AsmOperand::Imm(i),
                dst: AsmOperand::Register(AsmRegister::EAX),
            }],
            CExpression::Unary(_op, _exp) => todo!(),
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

        let codegen = AssemblyGenerator::new();

        let translated = codegen.translate(program);

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
