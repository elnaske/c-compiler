use crate::ir::ir_ast::*;
use std::collections::HashMap;
use std::vec;
pub mod asm_ast;
use asm_ast::*;

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
                AsmInstruction::Mov(op1, op2)
                | AsmInstruction::Binary(_, op1, op2)
                | AsmInstruction::Cmp(op1, op2) => {
                    self.pseudo_to_stack(op1, &mut curr_offset, &mut tmp_to_offset);
                    self.pseudo_to_stack(op2, &mut curr_offset, &mut tmp_to_offset);
                }
                AsmInstruction::Unary(_, op)
                | AsmInstruction::Idiv(op)
                | AsmInstruction::SetCC(_, op) => {
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
                    tmp_to_offset.insert(*tmp, *curr_offset);
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
        ir_instructions
            .into_iter()
            .flat_map(|ins| ins.to_asm())
            .collect()
    }

    pub fn generate_asm(&self, program: AsmProgram) -> String {
        let mut lines: Vec<String> = vec![
            format!("\t.globl {}", program.function.name),
            format!("{}:", program.function.name),
            "pushq %rbp".to_string(),
            "movq %rsp, %rbp".to_string(),
        ];

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
