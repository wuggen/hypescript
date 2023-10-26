//! The HypeScript abstract syntax tree structure

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinopSym {
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Eq,
    NEq,
    BitAnd,
    BitOr,
    BitXor,
    LogAnd,
    LogOr,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnopSym {
    BitNot,
    LogNot,
}

/// The abstract syntax tree.
#[derive(Debug, Clone)]
pub enum Ast {
    /// A block of statements
    Block(Vec<Ast>),

    /// Variable value
    Var(String),

    /// Unsigned int literal
    Int(u64),

    /// Boolean literal
    Boolean(bool),

    /// Assignment to a declared variable
    Assign { var: String, value: Box<Ast> },

    /// If statement, with optional else clauses
    IfCond {
        cond: Box<Ast>,
        body: Vec<Ast>,
        else_body: Vec<Ast>,
    },

    /// Binary operation
    Binop {
        sym: BinopSym,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },

    /// Unary operation
    Unop { sym: UnopSym, operand: Box<Ast> },

    /// Print statement
    Print(Box<Ast>),
}

impl Ast {
    /// Create a variable reference node.
    pub fn var(var: impl Into<String>) -> Self {
        Self::Var(var.into())
    }

    /// Create a variable assignment node.
    pub fn assign(var: impl Into<String>, value: Self) -> Self {
        Self::Assign {
            var: var.into(),
            value: Box::new(value),
        }
    }

    /// Create an if-else node.
    pub fn if_cond(cond: Self, body: Vec<Self>, else_body: Vec<Self>) -> Self {
        Self::IfCond {
            cond: Box::new(cond),
            body,
            else_body,
        }
    }

    /// Create a binary operator node.
    pub fn binop(sym: BinopSym, lhs: Self, rhs: Self) -> Self {
        Self::Binop {
            sym,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Create a unary operator node.
    pub fn unop(sym: UnopSym, operand: Self) -> Self {
        Self::Unop {
            sym,
            operand: Box::new(operand),
        }
    }

    /// Create a print node.
    pub fn print(val: Self) -> Self {
        Self::Print(Box::new(val))
    }
}

macro_rules! binop_fn {
    ($($fname:ident $Sym:ident $docstr:literal),*) => {
        $(
        #[doc = concat!("Create a ", $docstr, " node.")]
        pub fn $fname(lhs: Self, rhs: Self) -> Self {
            Self::binop(BinopSym::$Sym, lhs, rhs)
        }
        )*
    }
}

#[allow(clippy::should_implement_trait)]
impl Ast {
    binop_fn! {
        plus Plus "addition",
        minus Minus "subtraction",
        mul Mul "multiplication",
        div Div "division",
        mod_ Mod "modulo",
        greater Greater "greater-than comparison",
        less Less "less-than comparison",
        greater_eq GreaterEq "greater-or-equal comparison",
        less_eq LessEq "less-or-equal comparison",
        eq Eq "equality comparison",
        bit_and BitAnd "bitwise AND",
        bit_or BitOr "bitwise OR",
        bit_xor BitXor "bitwise XOR",
        log_and LogAnd "logical AND",
        log_or LogOr "logical OR"
    }

    /// Create a bitwise NOT node.
    pub fn bit_not(operand: Self) -> Self {
        Self::unop(UnopSym::BitNot, operand)
    }

    /// Create a logical NOT node.
    pub fn log_not(operand: Self) -> Self {
        Self::unop(UnopSym::LogNot, operand)
    }
}
