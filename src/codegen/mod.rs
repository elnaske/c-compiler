use crate::common::TempId;
use crate::ir::ir_ast::*;
use std::collections::HashMap;
use std::vec;
pub mod asm_ast;
use asm_ast::*;

fn round_to_16(n: usize) -> usize {
    (n + 15) & !15
}

pub struct AssemblyGenerator {
    fn_stack_sizes: HashMap<String, i32>,
}
impl Default for AssemblyGenerator {
    fn default() -> Self {
        Self::new()
    }
}
impl AssemblyGenerator {
    pub fn new() -> Self {
        AssemblyGenerator {
            fn_stack_sizes: HashMap::<String, i32>::new(),
        }
    }

    pub fn ir_to_asm(&mut self, ir_program: IRProgram) -> AsmProgram {
        let mut asm_program = self.translate_program(ir_program);
        self.replace_pseudo_registers(&mut asm_program);
        asm_program.functions = asm_program
            .functions
            .into_iter()
            .map(|f| AsmFunction {
                name: f.name.clone(),
                instructions: self.fix_instructions(
                    f.instructions,
                    self.fn_stack_sizes.get(&f.name).unwrap().unsigned_abs() as usize,
                ),
            })
            .collect();
        asm_program
    }

    fn replace_pseudo_registers(&mut self, asm_program: &mut AsmProgram) {
        let mut tmp_to_offset = HashMap::<TempId, i32>::new();
        let mut curr_offset = 0;

        for function in &mut asm_program.functions {
            for instruction in &mut function.instructions {
                match instruction {
                    AsmInstruction::Mov(op1, op2)
                    | AsmInstruction::Binary(_, op1, op2)
                    | AsmInstruction::Cmp(op1, op2) => {
                        self.pseudo_to_stack(op1, &mut curr_offset, &mut tmp_to_offset);
                        self.pseudo_to_stack(op2, &mut curr_offset, &mut tmp_to_offset);
                    }
                    AsmInstruction::Unary(_, op)
                    | AsmInstruction::Idiv(op)
                    | AsmInstruction::SetCC(_, op)
                    | AsmInstruction::Push(op) => {
                        self.pseudo_to_stack(op, &mut curr_offset, &mut tmp_to_offset);
                    }
                    _ => (),
                }
            }
            self.fn_stack_sizes
                .insert(function.name.clone(), curr_offset);
        }
    }

    fn pseudo_to_stack(
        &self,
        operand: &mut AsmOperand,
        curr_offset: &mut i32,
        tmp_to_offset: &mut HashMap<TempId, i32>,
    ) {
        if let AsmOperand::PseudoReg(tmp) = operand {
            let stack_offset = match tmp_to_offset.get(tmp) {
                Some(offset) => *offset,
                None => {
                    *curr_offset -= 4;
                    tmp_to_offset.insert(*tmp, *curr_offset);
                    *curr_offset
                }
            };
            *operand = AsmOperand::Stack(stack_offset);
        };
    }

    fn fix_instructions(
        &self,
        asm_instructions: Vec<AsmInstruction>,
        stack_size: usize,
    ) -> Vec<AsmInstruction> {
        std::iter::once(AsmInstruction::AllocateStack(round_to_16(stack_size)))
            .chain(asm_instructions.into_iter().flat_map(|ins| ins.fix()))
            .collect()
    }

    pub fn translate_program(&self, ir_program: IRProgram) -> AsmProgram {
        AsmProgram {
            functions: ir_program
                .functions
                .into_iter()
                .map(|f| self.translate_function(f))
                .collect(),
        }
    }

    fn translate_function(&self, ir_function: IRFunction) -> AsmFunction {
        let arg_registers = [
            AsmRegister::Edi,
            AsmRegister::Esi,
            AsmRegister::Edx,
            AsmRegister::Ecx,
            AsmRegister::R8d,
            AsmRegister::R9d,
        ];

        let mut instructions = Vec::<AsmInstruction>::new();
        for (i, id) in ir_function.param_ids.iter().enumerate() {
            match arg_registers.get(i) {
                // copy first 6 params from registers
                Some(reg) => {
                    instructions.push(AsmInstruction::Mov(
                        AsmOperand::Register(*reg),
                        AsmOperand::PseudoReg(*id),
                    ));
                }
                // copy remaining params from stack
                None => {
                    let offset = 16 + (i - arg_registers.len()) * 8;
                    instructions.push(AsmInstruction::Mov(
                        AsmOperand::Stack(offset as i32),
                        AsmOperand::PseudoReg(*id),
                    ));
                }
            }
        }

        instructions.append(&mut self.translate_instructions(ir_function.instructions));

        AsmFunction {
            name: ir_function.name,
            instructions,
        }
    }

    fn translate_instructions(&self, ir_instructions: Vec<IRInstruction>) -> Vec<AsmInstruction> {
        ir_instructions
            .into_iter()
            .flat_map(|ins| ins.to_asm())
            .collect()
    }

    pub fn generate_asm(&self, program: AsmProgram) -> String {
        let mut lines = Vec::<String>::new();
        for function in program.functions {
            let mut fn_setup: Vec<String> = vec![
                format!("\t.globl {}", function.name),
                format!("{}:", function.name),
                "\tpushq %rbp".to_string(),
                "\tmovq %rsp, %rbp".to_string(),
            ];
            lines.append(&mut fn_setup);
            lines.append(
                &mut function
                    .instructions
                    .into_iter()
                    .map(|instr| match instr {
                        AsmInstruction::Call(fn_name)
                            if self.fn_stack_sizes.contains_key(&fn_name) =>
                        {
                            let mut fn_name = fn_name.clone();
                            let _ = fn_name.split_off(fn_name.len() - 2);
                            format!("\tcall {}@PLT", fn_name)
                        }
                        _ => format!("\t{}", instr),
                    })
                    .collect::<Vec<String>>(),
            )
        }
        lines.push("\t.section .note.GNU-stack\n".to_string());

        lines.join("\n")
    }
}
