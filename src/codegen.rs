use std::vec;

use crate::parser;

#[derive(Debug, PartialEq)]
pub struct Program {
    function: Function,
}

#[derive(Debug, PartialEq)]
struct Function {
    name: String,
    instructions: Vec<Instruction>,
}

#[derive(Debug, PartialEq)]
enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}
impl Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Mov { src, dst } => {
                format!("\tmovl\t{}, {}", src.to_string(), dst.to_string())
            }
            Instruction::Ret => "\tret".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Operand {
    Imm(i32),
    Register(Register),
}
impl Operand {
    fn to_string(&self) -> String {
        match self {
            Operand::Imm(i) => format!("${}", i),
            Operand::Register(reg) => format!("%{}", reg.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Register {
    EAX,
}
impl Register {
    fn to_string(&self) -> String {
        match self {
            Register::EAX => "EAX".to_string(),
        }
    }
}

pub struct AssemblyGenerator {}
impl AssemblyGenerator {
    pub fn new() -> Self {
        AssemblyGenerator {}
    }

    pub fn translate(&self, c_program: parser::Program) -> Program {
        Program {
            function: self.translate_function(c_program.function),
        }
    }

    fn translate_function(&self, c_function: parser::Function) -> Function {
        Function {
            name: c_function.name,
            instructions: self.translate_statement(c_function.body),
        }
    }

    fn translate_statement(&self, statement: parser::Statement) -> Vec<Instruction> {
        let mut instructions: Vec<Instruction> = Vec::new();

        match statement {
            parser::Statement::Return(exp) => {
                instructions.append(&mut self.translate_expression(exp));
                instructions.push(Instruction::Ret);
            }
        }
        instructions
    }

    fn translate_expression(&self, expression: parser::Expression) -> Vec<Instruction> {
        match expression {
            parser::Expression::Constant(i) => vec![Instruction::Mov {
                src: Operand::Imm(i),
                dst: Operand::Register(Register::EAX),
            }],
        }
    }

    pub fn generate_asm(&self, program: Program) -> String {
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
    use std::process::Command;
    use tempfile::tempdir;
    use std::fs::File;

    #[test]
    fn return_2() {
        let code = b"int main(void) {
        return 2;
        }";

        let tokens = Lexer::new(code).get_tokens();

        let program = Parser::new(tokens).parse_program().unwrap();

        let codegen = AssemblyGenerator::new();

        let translated = codegen.translate(program);

        let ref_translation = Program {
            function: Function {
                name: "main".to_string(),
                instructions: vec![
                    Instruction::Mov {
                        src: Operand::Imm(2),
                        dst: Operand::Register(Register::EAX),
                    },
                    Instruction::Ret,
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

        let output = Command::new(exe_path)
            .output()
            .unwrap();

        assert_eq!(output.status.code().unwrap(), 2);
    }
}
