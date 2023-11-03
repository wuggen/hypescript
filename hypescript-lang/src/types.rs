//! The HypeScript type system and type checker.

use std::fmt::{self, Display, Formatter};

use crate::ast::{Ast, BinopSym, UnopSym};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    Unit,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Unit => write!(f, "Unit"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BinopClass {
    Int,
    Logical,
    Comp,
}

impl BinopClass {
    fn classify(op: BinopSym) -> Self {
        use BinopSym::*;
        match op {
            Plus | Minus | Mul | Div | Mod | BitAnd | BitOr | BitXor => BinopClass::Int,
            Greater | Less | GreaterEq | LessEq | Eq | NEq => BinopClass::Comp,
            LogAnd | LogOr => BinopClass::Logical,
        }
    }

    fn result_ty(self) -> Type {
        match self {
            Self::Int => Type::Int,
            Self::Logical | Self::Comp => Type::Bool,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TypeError {
    #[error("Found non-unit term before the end of a sequence (type found: {0})")]
    NonUnitInSequence(Type),

    #[error("Cannot bind variables to values of type Unit (variable name `{0}`)")]
    AssignUnitValue(String),

    #[error("Cannot re-bind variable `{name}` (type {ty}) to new type {new_ty}")]
    VariableTypeMismatch {
        name: String,
        ty: Type,
        new_ty: Type,
    },

    #[error("Undeclared variable `{0}`")]
    UndeclaredVariable(String),

    #[error("Invalid type for `if` condition: {0}")]
    InvalidConditionType(Type),

    #[error("Cannot yield non-unit type from bare `if` statement (found {0})")]
    NonUnitBareIfStatement(Type),

    #[error(
        "All clauses in an `if` statement must be of the same type (found {if_ty} and {else_ty})"
    )]
    MismatchedIfElseTypes { if_ty: Type, else_ty: Type },

    #[error("Expected operand of type {expected}, found {found}")]
    InvalidOperandType { expected: Type, found: Type },

    #[error("Cannot print value of type {0}; printed values must be integers or booleans")]
    InvalidPrintValueType(Type),
}

pub fn typecheck(ast: &[Ast]) -> Result<Type, TypeError> {
    let mut context = TypingContext::default();
    typecheck_sequence(&mut context, ast)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct TypingContext {
    vars: Vec<(String, Type)>,
}

impl TypingContext {
    fn lookup(&self, var: &str) -> Option<Type> {
        self.vars
            .iter()
            .rev()
            .find_map(|(name, ty)| if name == var { Some(*ty) } else { None })
    }

    fn bind(&mut self, var: String, ty: Type) -> Result<(), TypeError> {
        if let Some(old_ty) = self.lookup(&var) {
            if old_ty != ty {
                Err(TypeError::VariableTypeMismatch {
                    name: var,
                    ty: old_ty,
                    new_ty: ty,
                })
            } else {
                Ok(())
            }
        } else {
            self.vars.push((var, ty));
            Ok(())
        }
    }

    fn in_new_scope<T>(
        &self,
        f: impl FnOnce(&mut TypingContext) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        let mut new_scope = self.clone();
        f(&mut new_scope)
    }
}

fn typecheck_sequence(context: &mut TypingContext, ast: &[Ast]) -> Result<Type, TypeError> {
    ast.iter().fold(Ok(Type::Unit), |prev_ty, next_statement| {
        let prev_ty = prev_ty?;
        if prev_ty != Type::Unit {
            Err(TypeError::NonUnitInSequence(prev_ty))
        } else {
            typecheck_one(context, next_statement)
        }
    })
}

fn typecheck_one(context: &mut TypingContext, ast: &Ast) -> Result<Type, TypeError> {
    match ast {
        Ast::Block(seq) => context.in_new_scope(|context| typecheck_sequence(context, seq)),

        Ast::Var(v) => {
            if let Some(ty) = context.lookup(v) {
                Ok(ty)
            } else {
                Err(TypeError::UndeclaredVariable(v.clone()))
            }
        }

        Ast::Int(_) => Ok(Type::Int),

        Ast::Boolean(_) => Ok(Type::Bool),

        Ast::Assign { var, value } => {
            let ty = typecheck_one(context, value)?;
            if ty == Type::Unit {
                Err(TypeError::AssignUnitValue(var.clone()))
            } else {
                context.bind(var.clone(), ty)?;
                Ok(Type::Unit)
            }
        }

        Ast::IfCond {
            cond,
            body,
            else_body,
        } => {
            let cond_ty = typecheck_one(context, cond)?;
            if cond_ty != Type::Bool {
                Err(TypeError::InvalidConditionType(cond_ty))
            } else {
                let body_ty = context.in_new_scope(|context| typecheck_sequence(context, body))?;

                if else_body.is_empty() {
                    if body_ty == Type::Unit {
                        Ok(Type::Unit)
                    } else {
                        Err(TypeError::NonUnitBareIfStatement(body_ty))
                    }
                } else {
                    let else_ty =
                        context.in_new_scope(|context| typecheck_sequence(context, else_body))?;

                    if body_ty == else_ty {
                        Ok(body_ty)
                    } else {
                        Err(TypeError::MismatchedIfElseTypes {
                            if_ty: body_ty,
                            else_ty,
                        })
                    }
                }
            }
        }

        Ast::Binop { sym, lhs, rhs } => {
            let op_class = BinopClass::classify(*sym);

            let operand_type = match op_class {
                BinopClass::Int | BinopClass::Comp => Type::Int,
                BinopClass::Logical => Type::Bool,
            };

            let lhs_type = typecheck_one(context, lhs)?;
            if lhs_type != operand_type {
                return Err(TypeError::InvalidOperandType {
                    expected: operand_type,
                    found: lhs_type,
                });
            }

            let rhs_type = typecheck_one(context, rhs)?;
            if rhs_type != operand_type {
                return Err(TypeError::InvalidOperandType {
                    expected: operand_type,
                    found: rhs_type,
                });
            }

            Ok(op_class.result_ty())
        }

        Ast::Unop { sym, operand } => {
            let expected_type = match sym {
                UnopSym::BitNot => Type::Int,
                UnopSym::LogNot => Type::Bool,
            };

            let found_type = typecheck_one(context, operand)?;
            if found_type != expected_type {
                Err(TypeError::InvalidOperandType {
                    expected: expected_type,
                    found: found_type,
                })
            } else {
                Ok(expected_type)
            }
        }

        Ast::Print(value) => {
            let val_type = typecheck_one(context, value)?;
            if matches!(val_type, Type::Int | Type::Bool) {
                Ok(Type::Unit)
            } else {
                Err(TypeError::InvalidPrintValueType(val_type))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse;

    use super::*;

    fn test_typecheck(expected: Result<Type, TypeError>, input: &str) {
        let ast = parse::parse(input).expect("Parsing failed");

        assert_eq!(typecheck(&ast), expected);
    }

    #[test]
    fn literals() {
        test_typecheck(Ok(Type::Int), "45");
        test_typecheck(Ok(Type::Bool), "true");
        test_typecheck(Ok(Type::Bool), "false");
    }

    #[test]
    fn binops() {
        test_typecheck(Ok(Type::Int), "4 + 8");
        test_typecheck(Ok(Type::Int), "0x45 ^ 0x8f");
        test_typecheck(Ok(Type::Bool), "4 == 5");
        test_typecheck(Ok(Type::Bool), "8 <= 70");
        test_typecheck(Ok(Type::Bool), "(2 < 3) || (8 > 4)");
    }

    #[test]
    fn binops_error() {
        test_typecheck(
            Err(TypeError::InvalidOperandType {
                expected: Type::Int,
                found: Type::Bool,
            }),
            "4 + false",
        );

        test_typecheck(
            Err(TypeError::InvalidOperandType {
                expected: Type::Bool,
                found: Type::Int,
            }),
            "(4 == 5) || (3 + 2)",
        );

        test_typecheck(
            Err(TypeError::InvalidOperandType {
                expected: Type::Bool,
                found: Type::Unit,
            }),
            "true && { print 6; }",
        );
    }

    #[test]
    fn sequence() {
        test_typecheck(Ok(Type::Unit), "a = 4; b = false; print a == 5 || b;");
        test_typecheck(Ok(Type::Bool), "a = 6; b = 12; a > b");
        test_typecheck(Ok(Type::Int), "a = false; if a { 82 } else { 97 }");
    }

    #[test]
    fn sequence_extraneous_values() {
        test_typecheck(
            Err(TypeError::NonUnitInSequence(Type::Int)),
            "a = 4; (8 + 7) print a;",
        );
    }

    #[test]
    fn sequence_unbound_vars() {
        test_typecheck(
            Err(TypeError::UndeclaredVariable("a".into())),
            "b = 4; print a == b;",
        );
    }

    #[test]
    fn sequence_reassignment() {
        test_typecheck(Ok(Type::Unit), "a = 45; b = 12; a = a + b; print a;");
        test_typecheck(Ok(Type::Bool), "a = false; b = 3; if b < 4 { a = true; } a");
    }

    #[test]
    fn sequence_reassignment_type_mismatch() {
        test_typecheck(
            Err(TypeError::VariableTypeMismatch {
                name: "a".into(),
                ty: Type::Int,
                new_ty: Type::Bool,
            }),
            "a = 4; b = 5; a = b >= a;",
        );
    }

    #[test]
    fn assign_unit() {
        test_typecheck(
            Err(TypeError::AssignUnitValue("a".into())),
            "b = 4; a = if b == 0 { print 6; };",
        );
    }

    #[test]
    fn bare_if_non_unit() {
        test_typecheck(
            Err(TypeError::NonUnitBareIfStatement(Type::Int)),
            "if true { 4 }",
        );

        test_typecheck(
            Err(TypeError::NonUnitBareIfStatement(Type::Bool)),
            "a = 4; b = 5; if a == b { false }",
        );
    }

    #[test]
    fn if_else() {
        test_typecheck(Ok(Type::Int), "if true { 45 } else { a = 7; a + 6 }");
        test_typecheck(
            Ok(Type::Bool),
            "a = 80; if a < 40 { a == 20 } else { a > 100 }",
        );
        test_typecheck(Ok(Type::Unit), "if 4 < 9 { print 1; } else { print 0; }");
    }

    #[test]
    fn if_else_mismatch() {
        test_typecheck(
            Err(TypeError::MismatchedIfElseTypes {
                if_ty: Type::Unit,
                else_ty: Type::Int,
            }),
            "if false { print 6; } else { 8 - 2 }",
        );

        test_typecheck(
            Err(TypeError::MismatchedIfElseTypes {
                if_ty: Type::Int,
                else_ty: Type::Bool,
            }),
            "a = 5; b = 10; if a == b { b + 8 } else { b < 20 }",
        );
    }

    #[test]
    fn if_else_chain() {
        test_typecheck(
            Ok(Type::Unit),
            r#"a = 4;
b = 9;
if a < b {
    print 0;
} else if a == b {
    print 1;
} else {
    print 2;
}"#,
        );

        test_typecheck(
            Ok(Type::Int),
            r#"a = false;
b = 45;
if a {
    b * 14
} else if b > 13 {
    a = true;
    b + 1
} else {
    87
}"#,
        );
    }

    #[test]
    fn if_else_chain_non_unit() {
        test_typecheck(
            Err(TypeError::NonUnitBareIfStatement(Type::Int)),
            "if false { 45 } else if true { 86 }",
        );
    }

    #[test]
    fn if_else_chain_mismatch() {
        test_typecheck(
            Err(TypeError::MismatchedIfElseTypes {
                if_ty: Type::Int,
                else_ty: Type::Unit,
            }),
            "if true { a = 4; } else if false { 17 % 4 } else { print false; }",
        );
    }

    #[test]
    fn var_scope() {
        test_typecheck(
            Ok(Type::Unit),
            "a = 4; { b = 86; print a - b; } { b = (a < 12); print b; }",
        );

        test_typecheck(
            Ok(Type::Unit),
            "a = 4; if a - 2 == 0 { b = 3; print a == b; } else { b = true; print b || a > 3; }",
        );

        test_typecheck(Err(TypeError::UndeclaredVariable("a".into())), "a = a + 5;");

        test_typecheck(
            Err(TypeError::UndeclaredVariable("b".into())),
            "a = 4; { b = a + 5; } { print b; }",
        );
    }
}
