use crate::lexer::UnaryOp;
use crate::parser;

#[derive(Debug, PartialEq)]
pub struct Program {
    function: Function,
}
impl Program {
    pub fn from_c(c_program: parser::Program) -> Self {
        Program {
            function: Function::from_c(c_program.function),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Function {
    name: String,
    instructions: Vec<Instruction>,
}
impl Function {
    pub fn from_c(c_function: parser::Function) -> Self {
        Function { name: c_function.name, instructions: statement_to_instructions(c_function.body) }
    }
}

#[derive(Debug, PartialEq)]
enum Instruction {
    Return(Val),
    Unary { op: UnaryOp, src: Val, dst: Val }, // TODO: separate UnaryOp enum for IR
}

#[derive(Debug, PartialEq, Clone)]
enum Val {
    Constant(i32),
    Var(String),
}

fn statement_to_instructions(c_statement: parser::Statement) -> Vec<Instruction> {
    let mut instructions = Vec::<Instruction>::new();

    match c_statement {
        parser::Statement::Return(parser::Expression::Constant(i)) => instructions.push(Instruction::Return(Val::Constant(i))),
        parser::Statement::Return(exp) => {
            let (return_val, mut exp_instructions) = exp_to_instructions(exp);
            instructions.append(&mut exp_instructions);
            instructions.push(Instruction::Return(return_val));
        }
    }
    instructions
}

fn exp_to_instructions(c_expression: parser::Expression) -> (Val, Vec<Instruction>) {
    let val;
    let mut instructions = Vec::<Instruction>::new();
    
    match c_expression {
        parser::Expression::Constant(i) => val = Val::Constant(i),
        parser::Expression::Unary(op, inner_exp) => {
            let (src, mut inner_instructions) = exp_to_instructions(*inner_exp);
            let dst = Val::Var(create_temp_var_name());
            
            val = dst.clone();
            instructions.append(&mut inner_instructions);
            instructions.push(Instruction::Unary { op, src, dst });
        }
    }
    (val, instructions)
}

fn create_temp_var_name() -> String {
    todo!()
}