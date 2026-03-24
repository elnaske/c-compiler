use crate::lexer::UnaryOp;
use crate::parser::*;

#[derive(Debug, PartialEq)]
pub struct IRProgram {
    function: IRFunction,
}

#[derive(Debug, PartialEq)]
struct IRFunction {
    name: String,
    instructions: Vec<IRInstruction>,
}

#[derive(Debug, PartialEq)]
enum IRInstruction {
    Return(Val),
    Unary { op: UnaryOp, src: Val, dst: Val }, // TODO: separate UnaryOp enum for IR
}

#[derive(Debug, PartialEq, Clone)]
enum Val {
    Constant(i32),
    Var(TempId),
}

#[derive(Debug, PartialEq, Clone)]
struct TempId(usize);

pub struct IRGenerator {
    next_var_id: usize,
}
impl IRGenerator {
    pub fn new() -> Self {
        IRGenerator { next_var_id: 0 }
    }

    fn create_temp_var(&mut self) -> TempId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TempId(id)
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

    fn exp_to_instructions(&mut self, c_expression: CExpression) -> (Val, Vec<IRInstruction>) {
        let val;
        let mut instructions = Vec::<IRInstruction>::new();

        match c_expression {
            CExpression::Constant(i) => val = Val::Constant(i),
            CExpression::Unary(op, inner_exp) => {
                let (src, mut inner_instructions) = self.exp_to_instructions(*inner_exp);
                let dst = Val::Var(self.create_temp_var());

                val = dst.clone();
                instructions.append(&mut inner_instructions);
                instructions.push(IRInstruction::Unary { op, src, dst });
            }
        }
        (val, instructions)
    }
}
