pub mod ir_ast;
use crate::common::BinaryOp;
use crate::parser::c_ast::*;
use ir_ast::*;

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
