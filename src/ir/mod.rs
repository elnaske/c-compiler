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
        Self::new(0, 0)
    }
}
impl IRGenerator {
    pub fn new(next_var_id: u32, next_label_id: u32) -> Self {
        IRGenerator {
            next_var_id,
            next_label_id,
        }
    }

    fn create_temp_var(&mut self) -> TempId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TempId(id)
    }

    fn create_jump_label(&mut self, kind: LabelKind) -> Label {
        let id = self.next_label_id;
        self.next_label_id += 1;
        Label { kind, id }
    }

    fn resolve_loop_labels(&mut self, start: Option<Label>) -> (Label, Label, Label) {
        let start = start.expect("Encountered unlabeled loop");
        let continue_label = Label {
            kind: LabelKind::Continue,
            id: start.id,
        };
        let break_label = Label {
            kind: LabelKind::Break,
            id: start.id,
        };

        (start, continue_label, break_label)
    }

    pub fn c_to_ir(&mut self, c_program: CProgram) -> IRProgram {
        IRProgram {
            functions: c_program
                .functions
                .into_iter()
                .filter(|f| f.body.is_some()) // discard declarations w/o definitions
                .map(|f| self.translate_function(f))
                .collect(),
        }
    }

    fn translate_function(&mut self, c_function: CFnDecl) -> IRFunction {
        IRFunction {
            name: c_function.name,
            params: c_function
                .params
                .clone()
                .into_iter()
                .map(|p| p.name.expect("All non-Void parameters should have names"))
                .collect(),
            param_ids: c_function
                .params
                .into_iter()
                .map(|p| p.id.expect("Unresolved id"))
                .collect(),
            instructions: self.translate_function_body(c_function.body.unwrap_or(CBlock(vec![]))),
        }
    }

    fn translate_function_body(&mut self, c_block: CBlock) -> Vec<IRInstruction> {
        c_block
            .0
            .into_iter()
            .flat_map(|b| self.translate_block_item(b))
            .chain(std::iter::once(IRInstruction::Return(IRVal::Constant(0)))) // append return 0 to the end to avoid undefined behavior
            .collect()
    }

    fn translate_block_item(&mut self, c_block_item: CBlockItem) -> Vec<IRInstruction> {
        match c_block_item {
            CBlockItem::Declaration(dec) => match dec {
                CDeclaration::VarDecl(vdec) => self.var_declaration_to_instructions(vdec),
                CDeclaration::FnDecl(fdec) => match fdec.body {
                    Some(block) => {
                        self.translate_function_body(block) // the type checker should prevent this, but I'm leaving it in regardless
                    }
                    None => vec![],
                },
            },
            CBlockItem::Statement(stmnt) => self.statement_to_instructions(stmnt),
        }
    }

    fn var_declaration_to_instructions(&mut self, var_decl: CVarDecl) -> Vec<IRInstruction> {
        match var_decl.init {
            Some(exp) => {
                let (result, mut instructions) = self.exp_to_instructions(exp);
                let ir_var = IRVal::Var(var_decl.var.id.expect("Encountered unresolved variable"));
                instructions.push(IRInstruction::Copy(result, ir_var));
                instructions
            }
            None => vec![],
        }
    }

    fn statement_to_instructions(&mut self, c_statement: CStatement) -> Vec<IRInstruction> {
        match c_statement {
            CStatement::Return(exp) => {
                let (return_val, mut instructions) = self.exp_to_instructions(exp);
                instructions.push(IRInstruction::Return(return_val));
                instructions
            }
            CStatement::Expression(exp) => {
                let (_, instructions) = self.exp_to_instructions(exp);
                instructions
            }
            CStatement::If(cond, then, else_) => {
                let (cond_val, mut cond_instructions) = self.exp_to_instructions(cond);
                let mut then_instructions = self.statement_to_instructions(*then);
                let end_label = self.create_jump_label(LabelKind::End);

                match else_ {
                    Some(else_stmnt) => {
                        let mut else_instructions = self.statement_to_instructions(*else_stmnt);
                        let else_label = self.create_jump_label(LabelKind::Else);

                        cond_instructions.push(IRInstruction::JumpIfZero(cond_val, else_label));
                        cond_instructions.append(&mut then_instructions);
                        cond_instructions.push(IRInstruction::Jump(end_label));
                        cond_instructions.push(IRInstruction::Label(else_label));
                        cond_instructions.append(&mut else_instructions);
                    }
                    None => {
                        cond_instructions.push(IRInstruction::JumpIfZero(cond_val, end_label));
                        cond_instructions.append(&mut then_instructions);
                    }
                }
                cond_instructions.push(IRInstruction::Label(end_label));
                cond_instructions
            }
            CStatement::Compound(block) => block
                .0
                .into_iter()
                .flat_map(|x| self.translate_block_item(x))
                .collect(),
            CStatement::Break(label) => {
                vec![IRInstruction::Jump(
                    label.expect("Encountered unlabeled break statement"),
                )]
            }
            CStatement::Continue(label) => {
                vec![IRInstruction::Jump(
                    label.expect("Encountered unlabeled continue statement"),
                )]
            }
            CStatement::While(cond, body, start) => {
                let (_, continue_label, break_label) = self.resolve_loop_labels(start);
                let (cond_val, mut cond_instructions) = self.exp_to_instructions(cond);

                let mut instructions = vec![IRInstruction::Label(continue_label)];
                instructions.append(&mut cond_instructions);
                instructions.push(IRInstruction::JumpIfZero(cond_val, break_label));
                instructions.append(&mut self.statement_to_instructions(*body));
                instructions.push(IRInstruction::Jump(continue_label));
                instructions.push(IRInstruction::Label(break_label));

                instructions
            }
            CStatement::DoWhile(body, cond, start) => {
                let (start, continue_label, break_label) = self.resolve_loop_labels(start);
                let (cond_val, mut cond_instructions) = self.exp_to_instructions(cond);

                let mut instructions = vec![IRInstruction::Label(start)];
                instructions.append(&mut self.statement_to_instructions(*body));
                instructions.push(IRInstruction::Label(continue_label));
                instructions.append(&mut cond_instructions);
                instructions.push(IRInstruction::JumpIfNotZero(cond_val, start));
                instructions.push(IRInstruction::Label(break_label));

                instructions
            }
            CStatement::For(init, cond, post, body, start) => {
                let (start, continue_label, break_label) = self.resolve_loop_labels(start);

                let mut instructions = match init {
                    CForInit::InitDecl(dec) => self.var_declaration_to_instructions(dec),
                    CForInit::InitExp(exp) => match exp {
                        Some(e) => {
                            let (_, instructions) = self.exp_to_instructions(e);
                            instructions
                        }
                        None => vec![],
                    },
                };
                instructions.push(IRInstruction::Label(start));
                if let Some(c) = cond {
                    let (cond_val, mut cond_instructions) = self.exp_to_instructions(c);
                    instructions.append(&mut cond_instructions);
                    instructions.push(IRInstruction::JumpIfZero(cond_val, break_label));
                }
                instructions.append(&mut self.statement_to_instructions(*body));
                instructions.push(IRInstruction::Label(continue_label));
                if let Some(p) = post {
                    let (_, mut post_instructions) = self.exp_to_instructions(p);
                    instructions.append(&mut post_instructions);
                }
                instructions.push(IRInstruction::Jump(start));
                instructions.push(IRInstruction::Label(break_label));
                instructions
            }
            CStatement::Null => vec![],
        }
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
                let res = IRVal::Var(self.create_temp_var());

                let instructions = {
                    let (cond_val, cond_instructions) = self.exp_to_instructions(*cond);
                    let (then_val, then_instructions) = self.exp_to_instructions(*exp1);
                    let (else_val, else_instructions) = self.exp_to_instructions(*exp2);

                    let else_label = self.create_jump_label(LabelKind::Else);
                    let end_label = self.create_jump_label(LabelKind::End);

                    vec![
                        cond_instructions,
                        vec![IRInstruction::JumpIfZero(cond_val, else_label)],
                        then_instructions,
                        vec![
                            IRInstruction::Copy(then_val, res),
                            IRInstruction::Jump(end_label),
                            IRInstruction::Label(else_label),
                        ],
                        else_instructions,
                        vec![
                            IRInstruction::Copy(else_val, res),
                            IRInstruction::Label(end_label),
                        ],
                    ]
                }
                .into_iter()
                .flatten()
                .collect();

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
            CFactor::FunctionCall(name, args) => {
                let mut ir_args = Vec::<IRVal>::new();
                let mut instructions = Vec::<IRInstruction>::new();
                let dst = IRVal::Var(self.create_temp_var());

                for arg in args {
                    let (arg_val, mut arg_instructions) = self.exp_to_instructions(arg);
                    ir_args.push(arg_val);
                    instructions.append(&mut arg_instructions);
                }
                instructions.push(IRInstruction::FnCall(name, ir_args, dst));
                (dst, instructions)
            }
        }
    }

    fn binop_to_instructions(
        &mut self,
        op: BinaryOp,
        exp1: CExpression,
        exp2: CExpression,
    ) -> (IRVal, Vec<IRInstruction>) {
        let (src1, ins1) = self.exp_to_instructions(exp1);
        let (src2, ins2) = self.exp_to_instructions(exp2);
        let dst = IRVal::Var(self.create_temp_var());

        let instructions = match op {
            BinaryOp::LogicalAnd => {
                let false_label = self.create_jump_label(LabelKind::False);
                let end_label = self.create_jump_label(LabelKind::End);

                vec![
                    ins1,
                    vec![IRInstruction::JumpIfZero(src1, false_label)],
                    ins2,
                    vec![
                        IRInstruction::JumpIfZero(src2, false_label),
                        IRInstruction::Copy(IRVal::Constant(1), dst),
                        IRInstruction::Jump(end_label),
                        IRInstruction::Label(false_label),
                        IRInstruction::Copy(IRVal::Constant(0), dst),
                        IRInstruction::Label(end_label),
                    ],
                ]
            }
            BinaryOp::LogicalOr => {
                let true_label = self.create_jump_label(LabelKind::True);
                let end_label = self.create_jump_label(LabelKind::End);

                vec![
                    ins1,
                    vec![IRInstruction::JumpIfNotZero(src1, true_label)],
                    ins2,
                    vec![
                        IRInstruction::JumpIfNotZero(src2, true_label),
                        IRInstruction::Copy(IRVal::Constant(0), dst),
                        IRInstruction::Jump(end_label),
                        IRInstruction::Label(true_label),
                        IRInstruction::Copy(IRVal::Constant(1), dst),
                        IRInstruction::Label(end_label),
                    ],
                ]
            }
            _ => {
                vec![ins1, ins2, vec![IRInstruction::Binary(op, src1, src2, dst)]]
            }
        }
        .into_iter()
        .flatten()
        .collect();

        (dst, instructions)
    }
}
