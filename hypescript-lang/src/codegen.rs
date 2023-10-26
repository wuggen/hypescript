//! Code generation from ASTs.

use hypescript_bytecode::{Instruction, Opcode};

use crate::ast::{Ast, BinopSym, UnopSym};

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("Undeclared variable `{0}`")]
    UndeclaredVariable(String),
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    vars: Vec<String>,
    max_vars: usize,
}

impl Context {
    pub fn assign_var(&mut self, var: &str) -> usize {
        self.index_of(var).unwrap_or_else(|| {
            self.vars.push(var.into());
            self.max_vars += 1;
            self.vars.len() - 1
        })
    }

    pub fn index_of(&self, var: &str) -> Option<usize> {
        self.vars.iter().rposition(|s| s == var)
    }

    pub fn update_max_vars(&mut self, inner_ctx: &Context) {
        self.max_vars = self.max_vars.max(inner_ctx.max_vars);
    }

    pub fn in_new_scope<F, T>(&mut self, op: F) -> T
    where
        F: FnOnce(&mut Context) -> T,
    {
        let mut inner_ctx = self.clone();
        let res = op(&mut inner_ctx);
        self.update_max_vars(&inner_ctx);
        res
    }
}

pub fn translate(program: &[Ast]) -> Result<Vec<Instruction>, CodegenError> {
    let mut instructions = vec![
        Instruction::from(Opcode::Push8),
        Instruction::from(Opcode::VarRes),
    ];

    let mut ctx = Context::default();

    translate_sequence(&mut ctx, &mut instructions, program)?;

    instructions[0] = Instruction::optimal_push(ctx.max_vars as u64);
    Ok(instructions)
}

fn translate_sequence(
    ctx: &mut Context,
    instructions: &mut Vec<Instruction>,
    seq: &[Ast],
) -> Result<(), CodegenError> {
    for ast in seq {
        translate_one(ctx, instructions, ast)?;
    }

    Ok(())
}

fn translate_one(
    ctx: &mut Context,
    instructions: &mut Vec<Instruction>,
    ast: &Ast,
) -> Result<(), CodegenError> {
    match ast {
        Ast::Block(seq) => {
            ctx.in_new_scope(|ctx| {
                translate_sequence(ctx, instructions, seq)
            })
        }

        Ast::Var(var) => {
            let idx = ctx
                .index_of(var)
                .ok_or_else(|| CodegenError::UndeclaredVariable(var.clone()))?;
            instructions.extend_from_slice(&[
                Instruction::optimal_push(idx as u64),
                Instruction::from(Opcode::VarLd),
            ]);
            Ok(())
        }

        Ast::Int(val) => {
            instructions.push(Instruction::optimal_push(*val));
            Ok(())
        }

        Ast::Boolean(val) => {
            instructions.push(Instruction::optimal_push(*val as u64));
            Ok(())
        }

        Ast::Assign { var, value } => {
            translate_one(ctx, instructions, value)?;
            let idx = ctx.assign_var(var);
            instructions.extend_from_slice(&[
                Instruction::optimal_push(idx as u64),
                Instruction::from(Opcode::VarSt),
            ]);
            Ok(())
        }

        Ast::IfCond {
            cond,
            body,
            else_body,
        } => {
            translate_one(ctx, instructions, cond)?;

            let mut if_instrs = Vec::new();
            ctx.in_new_scope(|ctx| {
                translate_sequence(ctx, &mut if_instrs, body)
            })?;

            let mut else_instrs = Vec::new();
            ctx.in_new_scope(|ctx| {
                translate_sequence(ctx, &mut else_instrs, else_body)
            })?;

            let else_body_len = Instruction::combined_len(&else_instrs);
            if else_body_len > 0 {
                if_instrs.extend_from_slice(&[
                    Instruction::optimal_pushs(else_body_len as i64),
                    Instruction::from(Opcode::Jump),
                ]);

                if_instrs.append(&mut else_instrs);
            }

            let if_len = Instruction::combined_len(&if_instrs);
            instructions.extend_from_slice(&[
                Instruction::from(Opcode::Not),
                Instruction::optimal_pushs(if_len as i64),
                Instruction::from(Opcode::JCond),
            ]);

            Ok(())
        }

        Ast::Binop { sym, lhs, rhs } => {
            translate_one(ctx, instructions, lhs)?;
            translate_one(ctx, instructions, rhs)?;
            append_binop_instrs(instructions, *sym);
            Ok(())
        }

        Ast::Unop { sym, operand } => {
            translate_one(ctx, instructions, operand)?;
            append_unop_instrs(instructions, *sym);
            Ok(())
        }

        Ast::Print(val) => {
            translate_one(ctx, instructions, val)?;
            instructions.push(Instruction::from(Opcode::Print));
            Ok(())
        }
    }
}

fn append_binop_instrs(instrs: &mut Vec<Instruction>, op: BinopSym) {
    match op {
        BinopSym::Plus => instrs.push(Instruction::from(Opcode::Add)),
        BinopSym::Minus => instrs.push(Instruction::from(Opcode::Sub)),
        BinopSym::Mul => instrs.push(Instruction::from(Opcode::Mul)),
        BinopSym::Div => instrs.push(Instruction::from(Opcode::Div)),
        BinopSym::Mod => instrs.push(Instruction::from(Opcode::Mod)),
        BinopSym::Greater => instrs.push(Instruction::from(Opcode::Gt)),
        BinopSym::Less => instrs.push(Instruction::from(Opcode::Lt)),
        BinopSym::GreaterEq => instrs.push(Instruction::from(Opcode::Ge)),
        BinopSym::LessEq => instrs.push(Instruction::from(Opcode::Le)),
        BinopSym::Eq => instrs.push(Instruction::from(Opcode::Eq)),
        BinopSym::NEq => instrs.extend_from_slice(&[
            Instruction::from(Opcode::Eq),
            Instruction::from(Opcode::Not),
        ]),
        BinopSym::BitAnd | BinopSym::LogAnd => instrs.push(Instruction::from(Opcode::And)),
        BinopSym::BitOr | BinopSym::LogOr => instrs.push(Instruction::from(Opcode::Or)),
        BinopSym::BitXor => instrs.push(Instruction::from(Opcode::Xor)),
    };
}

fn append_unop_instrs(instrs: &mut Vec<Instruction>, op: UnopSym) {
    match op {
        UnopSym::BitNot => instrs.push(Instruction::from(Opcode::Inv)),
        UnopSym::LogNot => instrs.push(Instruction::from(Opcode::Not)),
    }
}
