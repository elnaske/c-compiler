use crate::lexer::UnaryOp;
use crate::parser::*;

#[derive(Debug, PartialEq)]
pub struct IRProgram {
    function: IRFunction,
}
impl IRProgram {
    pub fn from_c(c_program: CProgram) -> Self {
        IRProgram {
            function: IRFunction::from_c(c_program.function),
        }
    }
}

#[derive(Debug, PartialEq)]
struct IRFunction {
    name: String,
    instructions: Vec<IRInstruction>,
}
impl IRFunction {
    pub fn from_c(c_function: CFunction) -> Self {
        IRFunction { name: c_function.name, instructions: statement_to_instructions(c_function.body) }
    }
}

#[derive(Debug, PartialEq)]
enum IRInstruction {
    Return(Val),
    Unary { op: UnaryOp, src: Val, dst: Val }, // TODO: separate UnaryOp enum for IR
}

#[derive(Debug, PartialEq, Clone)]
enum Val {
    Constant(i32),
    Var(String),
}

// TODO: TempId struct instead of String
// TODO: IRGenerator struct
// TODO: Add IR to main
// TODO: Add IR compiler flag

fn statement_to_instructions(c_statement: CStatement) -> Vec<IRInstruction> {
    let mut instructions = Vec::<IRInstruction>::new();

    match c_statement {
        CStatement::Return(exp) => {
            let (return_val, mut exp_instructions) = exp_to_instructions(exp);
            instructions.append(&mut exp_instructions);
            instructions.push(IRInstruction::Return(return_val));
        }
    }
    instructions
}

fn exp_to_instructions(c_expression: CExpression) -> (Val, Vec<IRInstruction>) {
    let val;
    let mut instructions = Vec::<IRInstruction>::new();
    
    match c_expression {
        CExpression::Constant(i) => val = Val::Constant(i),
        CExpression::Unary(op, inner_exp) => {
            let (src, mut inner_instructions) = exp_to_instructions(*inner_exp);
            let dst = Val::Var(create_temp_var_name());
            
            val = dst.clone();
            instructions.append(&mut inner_instructions);
            instructions.push(IRInstruction::Unary { op, src, dst });
        }
    }
    (val, instructions)
}

fn create_temp_var_name() -> String {
    todo!()
}