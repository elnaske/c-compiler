pub mod ir_ast;
use crate::common::{BinaryOp, TempId};
use crate::parser::c_ast::*;
use ir_ast::*;

pub struct IRGenerator {
    next_var_id: u32,
    next_label_id: u32,
}
impl Default for IRGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}
impl IRGenerator {
    pub fn new(next_var_id: u32) -> Self {
        IRGenerator {
            next_var_id,
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
            instructions: c_function
                .body
                .into_iter()
                .flat_map(|b| self.translate_block_item(b))
                .chain(std::iter::once(IRInstruction::Return(IRVal::Constant(0)))) // append return 0 to the end to avoid undefined behavior
                .collect(),
        }
    }

    fn translate_block_item(&mut self, c_block_item: CBlockItem) -> Vec<IRInstruction> {
        match c_block_item {
            CBlockItem::Declaration(dec) => match dec.init {
                Some(exp) => {
                    let (result, mut instructions) = self.exp_to_instructions(exp);
                    let ir_var = IRVal::Var(dec.var.id.expect("IDK man"));
                    instructions.push(IRInstruction::Copy(result, ir_var));
                    instructions
                }
                None => vec![],
            },
            CBlockItem::Statement(stmnt) => self.statement_to_instructions(stmnt),
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
            CStatement::Expression(exp) => {
                let (_, mut exp_instructions) = self.exp_to_instructions(exp);
                instructions.append(&mut exp_instructions);
            }
            CStatement::If(cond, then, else_) => {
                let (cond_val, mut cond_instructions) = self.exp_to_instructions(cond);
                let mut then_instructions = self.statement_to_instructions(*then);
                let end_label = self.create_jump_label();

                instructions.append(&mut cond_instructions);

                match else_ {
                    Some(else_stmnt) => {
                        let mut else_instructions = self.statement_to_instructions(*else_stmnt);
                        let else_label = self.create_jump_label();

                        instructions.push(IRInstruction::JumpIfZero(cond_val, else_label));
                        instructions.append(&mut then_instructions);
                        instructions.push(IRInstruction::Jump(end_label));
                        instructions.push(IRInstruction::Label(else_label));
                        instructions.append(&mut else_instructions);
                    }
                    None => {
                        instructions.push(IRInstruction::JumpIfZero(cond_val, end_label));
                        instructions.append(&mut then_instructions);
                    }
                }
                instructions.push(IRInstruction::Label(end_label));
            }
            CStatement::Null => (),
        }
        instructions
    }

    fn exp_to_instructions(&mut self, c_expression: CExpression) -> (IRVal, Vec<IRInstruction>) {
        match c_expression {
            CExpression::Factor(f) => self.factor_to_instructions(*f),
            CExpression::Binary(op, exp1, exp2) => self.binop_to_instructions(op, *exp1, *exp2),
            CExpression::Assign(exp1, exp2) => {
                if let CExpression::Factor(f) = *exp1
                    && let CFactor::Var(var) = *f
                {
                    let (result, instructions) = self.exp_to_instructions(*exp2);
                    let ir_var = IRVal::Var(var.id.expect("Variable unresolved"));
                    (
                        ir_var,
                        instructions
                            .into_iter()
                            .chain(std::iter::once(IRInstruction::Copy(result, ir_var)))
                            .collect(),
                    )
                } else {
                    panic!("Looks like variable resolution has a bug lol");
                }
            }
            CExpression::Conditional(cond, exp1, exp2) => {
                let mut instructions = Vec::new();
                let (cond_val, mut cond_instructions) = self.exp_to_instructions(*cond);
                let else_label = self.create_jump_label();
                let end_label = self.create_jump_label();
                let res = IRVal::Var(self.create_temp_var());

                instructions.append(&mut cond_instructions);

                instructions.push(IRInstruction::JumpIfZero(cond_val, else_label));
                let (then_val, mut then_instructions) = self.exp_to_instructions(*exp1);
                instructions.append(&mut then_instructions);
                instructions.push(IRInstruction::Copy(then_val, res));
                instructions.push(IRInstruction::Jump(end_label));

                instructions.push(IRInstruction::Label(else_label));
                let (else_val, mut else_instructions) = self.exp_to_instructions(*exp2);
                instructions.append(&mut else_instructions);
                instructions.push(IRInstruction::Copy(else_val, res));
                instructions.push(IRInstruction::Label(end_label));

                (res, instructions)
            }
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
            CFactor::Expression(exp) => self.exp_to_instructions(exp),
            CFactor::Var(var) => (IRVal::Var(var.id.unwrap()), vec![]),
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
