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
        Ast::Block(seq) => ctx.in_new_scope(|ctx| translate_sequence(ctx, instructions, seq)),

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
            ctx.in_new_scope(|ctx| translate_sequence(ctx, &mut if_instrs, body))?;

            let mut else_instrs = Vec::new();
            ctx.in_new_scope(|ctx| translate_sequence(ctx, &mut else_instrs, else_body))?;

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
            instructions.append(&mut if_instrs);

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

#[cfg(test)]
mod test {
    use hypescript_bytecode::instructions_to_vec;
    use hypescript_vm::ExecutionContext;

    use super::*;

    #[test]
    fn example1() {
        // Example 1 from the assignment:
        //
        // a = 5;
        // b = 4 + a;
        // print a;
        // print b;

        let program = &[
            Ast::assign("a", Ast::Int(5)),
            Ast::assign("b", Ast::plus(Ast::Int(4), Ast::var("a"))),
            Ast::print(Ast::var("a")),
            Ast::print(Ast::var("b")),
        ];

        let instructions = translate(program).expect("Failed to translate AST");

        use Opcode::*;
        let expected = &[
            // Preamble, reserve variables
            Instruction::new(Push8, 2),
            Instruction::from(VarRes),
            // a = 5
            Instruction::new(Push8, 5),
            Instruction::new(Push8, 0),
            Instruction::from(VarSt),
            // b = 4 + a
            Instruction::new(Push8, 4),
            Instruction::new(Push8, 0),
            Instruction::from(VarLd),
            Instruction::from(Add),
            Instruction::new(Push8, 1),
            Instruction::from(VarSt),
            // print a
            Instruction::new(Push8, 0),
            Instruction::from(VarLd),
            Instruction::from(Print),
            // print b
            Instruction::new(Push8, 1),
            Instruction::from(VarLd),
            Instruction::from(Print),
        ];

        assert_eq!(expected, instructions.as_slice());

        let bytes = instructions_to_vec(&instructions);
        let mut output = Vec::new();
        let _summary = ExecutionContext::new(&bytes)
            .with_output_stream(&mut output)
            .with_trace()
            .run()
            .expect("Runtime error");

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "5\n9\n");
    }

    #[test]
    fn example2() {
        // Example 2 from the assignment:
        //
        // a = 1;
        // b = 0;
        // if b == a {
        //     print 0;
        // }
        // if a > b {
        //     print 2;
        // }
        // print a + b;

        let program = &[
            Ast::assign("a", Ast::Int(1)),
            Ast::assign("b", Ast::Int(0)),
            Ast::if_cond(
                Ast::eq(Ast::var("b"), Ast::var("a")),
                vec![Ast::print(Ast::Int(0))],
                vec![],
            ),
            Ast::if_cond(
                Ast::greater(Ast::var("a"), Ast::var("b")),
                vec![Ast::print(Ast::Int(2))],
                vec![],
            ),
            Ast::print(Ast::plus(Ast::var("a"), Ast::var("b"))),
        ];

        let instructions = translate(program).expect("Failed to translate AST");

        use Opcode::*;
        let expected = &[
            // Preamble: reserve vars
            Instruction::new(Push8, 2),
            Instruction::from(VarRes),
            // a = 1
            Instruction::new(Push8, 1),
            Instruction::new(Push8, 0),
            Instruction::from(VarSt),
            // b = 0
            Instruction::new(Push8, 0),
            Instruction::new(Push8, 1),
            Instruction::from(VarSt),
            // if b == a
            Instruction::new(Push8, 1),
            Instruction::from(VarLd),
            Instruction::new(Push8, 0),
            Instruction::from(VarLd),
            Instruction::from(Eq),
            Instruction::from(Not),
            Instruction::new(Push8S, 3),
            Instruction::from(JCond),
            // { print 0 }
            Instruction::new(Push8, 0),
            Instruction::from(Print),
            // if a > b
            Instruction::new(Push8, 0),
            Instruction::from(VarLd),
            Instruction::new(Push8, 1),
            Instruction::from(VarLd),
            Instruction::from(Gt),
            Instruction::from(Not),
            Instruction::new(Push8S, 3),
            Instruction::from(JCond),
            // { print 2 }
            Instruction::new(Push8, 2),
            Instruction::from(Print),
            // print a + b
            Instruction::new(Push8, 0),
            Instruction::from(VarLd),
            Instruction::new(Push8, 1),
            Instruction::from(VarLd),
            Instruction::from(Add),
            Instruction::from(Print),
        ];

        assert_eq!(
            expected,
            instructions.as_slice(),
            "Expected: {:#?}\nActual: {:#?}",
            expected,
            instructions
        );

        let bytes = instructions_to_vec(&instructions);
        let mut output = Vec::<u8>::new();
        let _summary = ExecutionContext::new(&bytes)
            .with_output_stream(&mut output)
            .with_trace()
            .run()
            .expect("Runtime error");

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "2\n1\n");
    }
}
