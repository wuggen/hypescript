//! A parser for HypeScript.

use std::fmt::{self, Display, Formatter};

use chumsky::prelude::*;

use crate::ast::{Ast, BinopSym, UnopSym};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tok {
    Kw(Kw),
    Bool(bool),
    Ident(String),
    HexInt(String),
    DecInt(String),
    Binop(BinopSym),
    Unop(UnopSym),
    Punct(Punct),
}

impl Display for Tok {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Kw(kw) => write!(f, "{kw}"),
            Tok::Bool(b) => write!(f, "{b}"),
            Tok::Ident(id) => write!(f, "{id}"),
            Tok::HexInt(n) => write!(f, "{n}"),
            Tok::DecInt(n) => write!(f, "{n}"),
            Tok::Binop(op) => write!(f, "{op}"),
            Tok::Unop(op) => write!(f, "{op}"),
            Tok::Punct(punct) => write!(f, "{punct}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kw {
    If,
    Else,
    Print,
}

impl Display for Kw {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Kw::If => write!(f, "if"),
            Kw::Else => write!(f, "else"),
            Kw::Print => write!(f, "print"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punct {
    Semi,
    Eq,
    OBrace,
    CBrace,
    OParen,
    CParen,
}

impl Display for Punct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Punct::Semi => write!(f, ";"),
            Punct::Eq => write!(f, "="),
            Punct::OBrace => write!(f, "{{"),
            Punct::CBrace => write!(f, "}}"),
            Punct::OParen => write!(f, "("),
            Punct::CParen => write!(f, ")"),
        }
    }
}

/// Binary operator binding strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingStrength {
    /// Weakly-binding logical operators: `||`
    LogWeak,

    /// Strongly-binding logical operators: `&&`
    LogStrong,

    /// Comparison operators: `>`, `>=`, `<`, `<=`, `==`, `!=`
    Comp,

    /// Weakly-binding arithmetic operators: `+`, `-`
    ArithWeak,

    /// Mid-level arithmetic operators: `&`, `|`, `^`
    ArithMid,

    /// Strongly-binding arithmetic operators: `*`, `/`, `%`
    ArithStrong,
}

impl BindingStrength {
    fn classify(sym: BinopSym) -> Self {
        use BindingStrength::*;

        match sym {
            BinopSym::Mul | BinopSym::Div | BinopSym::Mod => ArithStrong,
            BinopSym::BitOr | BinopSym::BitAnd | BinopSym::BitXor => ArithMid,
            BinopSym::Plus | BinopSym::Minus => ArithWeak,
            BinopSym::Eq
            | BinopSym::NEq
            | BinopSym::Greater
            | BinopSym::Less
            | BinopSym::GreaterEq
            | BinopSym::LessEq => Comp,
            BinopSym::LogAnd => LogStrong,
            BinopSym::LogOr => LogWeak,
        }
    }

    fn increment(self) -> Option<Self> {
        match self {
            BindingStrength::LogWeak => Some(Self::LogStrong),
            BindingStrength::LogStrong => Some(Self::Comp),
            BindingStrength::Comp => Some(Self::ArithWeak),
            BindingStrength::ArithWeak => Some(Self::ArithMid),
            BindingStrength::ArithMid => Some(Self::ArithStrong),
            BindingStrength::ArithStrong => None,
        }
    }
}

fn ident_or_kw() -> impl Parser<char, Tok, Error = Simple<char>> {
    text::ident().map(|id: String| match id.as_str() {
        "if" => Tok::Kw(Kw::If),
        "else" => Tok::Kw(Kw::Else),
        "print" => Tok::Kw(Kw::Print),
        "true" => Tok::Bool(true),
        "false" => Tok::Bool(false),
        _ => Tok::Ident(id),
    })
}

fn int_tok() -> impl Parser<char, Tok, Error = Simple<char>> {
    let hex_int = just("0x").ignore_then(text::digits(16)).map(Tok::HexInt);
    let dec_int = text::digits(10).map(Tok::DecInt);
    hex_int.or(dec_int)
}

fn binop() -> impl Parser<char, Tok, Error = Simple<char>> {
    choice((
        just("+").to(BinopSym::Plus),
        just("-").to(BinopSym::Minus),
        just("*").to(BinopSym::Mul),
        just("/").to(BinopSym::Div),
        just("%").to(BinopSym::Mod),
        just(">=").to(BinopSym::GreaterEq),
        just(">").to(BinopSym::Greater),
        just("<=").to(BinopSym::LessEq),
        just("<").to(BinopSym::Less),
        just("==").to(BinopSym::Eq),
        just("!=").to(BinopSym::NEq),
        just("&&").to(BinopSym::LogAnd),
        just("&").to(BinopSym::BitAnd),
        just("||").to(BinopSym::LogOr),
        just("|").to(BinopSym::BitOr),
        just("^").to(BinopSym::BitXor),
    ))
    .map(Tok::Binop)
}

fn unop() -> impl Parser<char, Tok, Error = Simple<char>> {
    just("~")
        .to(UnopSym::BitNot)
        .or(just("!").to(UnopSym::LogNot))
        .map(Tok::Unop)
}

fn punct() -> impl Parser<char, Tok, Error = Simple<char>> {
    choice((
        just(";").to(Punct::Semi),
        just("=").to(Punct::Eq),
        just("{").to(Punct::OBrace),
        just("}").to(Punct::CBrace),
        just("(").to(Punct::OParen),
        just(")").to(Punct::CParen),
    ))
    .map(Tok::Punct)
}

fn comment() -> impl Parser<char, Option<Tok>, Error = Simple<char>> {
    let line_comment = just("//").ignore_then(take_until(just("\n")));
    let block_comment = just("/*").ignore_then(take_until(just("*/")));
    line_comment.or(block_comment).to(None)
}

pub fn lexer() -> impl Parser<char, Vec<Tok>, Error = Simple<char>> {
    let tok = choice((ident_or_kw(), int_tok(), binop(), unop(), punct()))
        .padded()
        .map(Some);

    text::whitespace()
        .ignore_then(comment().padded().or(tok).repeated())
        .flatten()
        .then_ignore(end())
}

fn statement<'a>(
    expr: Recursive<'a, Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Ast, Error = Simple<Tok>> + 'a {
    choice((assignment(expr.clone()), print(expr.clone()), expr))
}

fn assignment(
    expr: Recursive<Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Ast, Error = Simple<Tok>> + '_ {
    let var = filter_map(|span, tok| {
        if let Tok::Ident(v) = tok {
            Ok(v)
        } else {
            Err(Simple::custom(span, "expected variable name"))
        }
    });

    var.then_ignore(just(&[Tok::Punct(Punct::Eq)]))
        .then(expr)
        .then_ignore(just(&[Tok::Punct(Punct::Semi)]))
        .map(|(v, exp)| Ast::assign(v, exp))
}

fn print(
    expr: Recursive<Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Ast, Error = Simple<Tok>> + '_ {
    just(&[Tok::Kw(Kw::Print)])
        .ignore_then(expr)
        .then_ignore(just(&[Tok::Punct(Punct::Semi)]))
        .map(Ast::print)
}

fn if_chain<'a>(
    expr: Recursive<'a, Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Ast, Error = Simple<Tok>> + 'a {
    recursive(|if_chain| {
        let if_clause = just(&[Tok::Kw(Kw::If)])
            .ignore_then(expr.clone())
            .then(block(expr.clone()));

        let else_if_clauses = just(&[Tok::Kw(Kw::Else)])
            .ignore_then(if_chain.map(|ast| vec![ast]).or(block(expr)))
            .or_not();

        if_clause
            .then(else_if_clauses)
            .map(|((cond, body), else_clause)| {
                let else_body = else_clause.unwrap_or_else(|| vec![]);
                Ast::if_cond(cond, body, else_body)
            })
    })
}

fn seq<'a>(
    expr: Recursive<'a, Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Vec<Ast>, Error = Simple<Tok>> + 'a {
    statement(expr).repeated()
}

fn block(
    expr: Recursive<Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Vec<Ast>, Error = Simple<Tok>> + '_ {
    seq(expr).delimited_by(
        just(&[Tok::Punct(Punct::OBrace)]),
        just(&[Tok::Punct(Punct::CBrace)]),
    )
}

fn unop_factor(
    factor: Recursive<Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Ast, Error = Simple<Tok>> + '_ {
    let op = filter_map(|span, tok| {
        if let Tok::Unop(sym) = tok {
            Ok(sym)
        } else {
            Err(Simple::custom(span, "expected unary operator"))
        }
    });

    op.then(factor).map(|(sym, val)| Ast::unop(sym, val))
}

fn factor(
    expr: Recursive<Tok, Ast, Simple<Tok>>,
) -> impl Parser<Tok, Ast, Error = Simple<Tok>> + '_ {
    let lit_or_var = select! { |span|
        Tok::HexInt(s) => {
            let val = u64::from_str_radix(&s, 16).map_err(|e| Simple::custom(span, e))?;
            Ast::Int(val)
        },

        Tok::DecInt(s) => {
            let val = s.parse::<u64>().map_err(|e| Simple::custom(span, e))?;
            Ast::Int(val)
        },

        Tok::Bool(b) => Ast::Boolean(b),

        Tok::Ident(s) => Ast::Var(s),
    };

    recursive(|factor| {
        choice((
            lit_or_var,
            unop_factor(factor),
            expr.clone().delimited_by(
                just(&[Tok::Punct(Punct::OParen)]),
                just(&[Tok::Punct(Punct::CParen)]),
            ),
            if_chain(expr.clone()),
            block(expr).map(Ast::Block),
        ))
    })
}

fn expr_binop_strength(
    strength: BindingStrength,
    expr: Recursive<Tok, Ast, Simple<Tok>>,
) -> Box<dyn Parser<Tok, Ast, Error = Simple<Tok>> + '_> {
    let op = filter_map(move |span, tok| match tok {
        Tok::Binop(sym) => {
            let sym_strength = BindingStrength::classify(sym);
            if sym_strength != strength {
                Err(Simple::custom(
                    span,
                    format!("expected operator of binding strength {strength:?}"),
                ))
            } else {
                Ok(sym)
            }
        }
        _ => Err(Simple::custom(span, "expected binary operator")),
    });

    if let Some(next_strength) = strength.increment() {
        Box::new(
            expr_binop_strength(next_strength, expr.clone())
                .then(op.then(expr_binop_strength(next_strength, expr)).repeated())
                .foldl(|lhs, (sym, rhs)| Ast::binop(sym, lhs, rhs)),
        )
    } else {
        Box::new(
            factor(expr.clone())
                .then(op.then(factor(expr)).repeated())
                .foldl(|lhs, (sym, rhs)| Ast::binop(sym, lhs, rhs)),
        )
    }
}

fn expr() -> Recursive<'static, Tok, Ast, Simple<Tok>> {
    recursive(|expr| {
        expr_binop_strength(BindingStrength::LogWeak, expr)
    })
}

pub fn parser() -> impl Parser<Tok, Vec<Ast>, Error = Simple<Tok>> {
    seq(expr()).then_ignore(end())
}

pub fn parse(input: &str) -> Result<Vec<Ast>, Vec<Simple<Tok>>> {
    let toks = lexer().parse(input).map_err(|errs| {
        errs.into_iter()
            .map(|e| Simple::custom(e.span(), e))
            .collect::<Vec<_>>()
    })?;
    parser().parse(toks)
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_lexer(input: &str, expected: &[Tok]) {
        let toks = lexer().parse(input).expect("Lexer failed");
        assert_eq!(toks, expected);
    }

    fn test_parser(input: &str, expected: &[Ast]) {
        let toks = lexer().parse(input).expect("Lexer failed");
        eprintln!("{toks:?}");
        let ast = parser().parse(toks).expect("Parser failed");
        assert_eq!(ast, expected);
    }

    #[test]
    fn tok_empty() {
        test_lexer("", &[]);
        test_lexer("     ", &[]);
        test_lexer("\n\n\t   \t\n  ", &[]);
    }

    #[test]
    fn tok_comments() {
        test_lexer(
            r#"hey // this should be skipped
a + b /* this also */ more"#,
            &[
                Tok::Ident("hey".into()),
                Tok::Ident("a".into()),
                Tok::Binop(BinopSym::Plus),
                Tok::Ident("b".into()),
                Tok::Ident("more".into()),
            ],
        );

        test_lexer(
            "/*/",
            &[
                Tok::Binop(BinopSym::Div),
                Tok::Binop(BinopSym::Mul),
                Tok::Binop(BinopSym::Div),
            ],
        );

        // Block comments don't nest
        test_lexer("/* hey /* */ there", &[Tok::Ident("there".into())]);
    }

    macro_rules! binop_prefix_test {
        ($($op1:tt => $exp1:ident, $op2:tt => $exp2:ident ;)*) => {
            $(test_lexer(
                concat!(stringify!($op1), " ", stringify!($op2)),
                &[Tok::Binop(BinopSym::$exp1), Tok::Binop(BinopSym::$exp2)],
            );)*
        }
    }

    #[test]
    fn tok_operator_prefixes() {
        binop_prefix_test! {
            && => LogAnd, & => BitAnd;
            || => LogOr, | => BitOr;
            >= => GreaterEq, > => Greater;
            <= => LessEq, < => Less;
        }

        test_lexer(
            "!= !",
            &[Tok::Binop(BinopSym::NEq), Tok::Unop(UnopSym::LogNot)],
        );

        test_lexer("== =", &[Tok::Binop(BinopSym::Eq), Tok::Punct(Punct::Eq)]);
    }

    #[test]
    fn tok_int_literals() {
        test_lexer(
            "1792 1909 0x42325 0xaFFe5",
            &[
                Tok::DecInt("1792".into()),
                Tok::DecInt("1909".into()),
                Tok::HexInt("42325".into()),
                Tok::HexInt("aFFe5".into()),
            ],
        );
    }

    #[test]
    fn tok_keywords() {
        test_lexer(
            "if else print true false something_else if_not_kw",
            &[
                Tok::Kw(Kw::If),
                Tok::Kw(Kw::Else),
                Tok::Kw(Kw::Print),
                Tok::Bool(true),
                Tok::Bool(false),
                Tok::Ident("something_else".into()),
                Tok::Ident("if_not_kw".into()),
            ],
        );
    }

    #[test]
    fn parse_arithmetic() {
        // Weakly-binding operators are left-associative
        test_parser(
            "a + b - c",
            &[Ast::minus(
                Ast::plus(Ast::var("a"), Ast::var("b")),
                Ast::var("c"),
            )],
        );

        // Strongly-binding operators are, y'know, strongly binding
        test_parser(
            "a - b * c",
            &[Ast::minus(
                Ast::var("a"),
                Ast::mul(Ast::var("b"), Ast::var("c")),
            )],
        );

        // Strongly-binding operators are left-associative
        test_parser(
            "a / b * c",
            &[Ast::mul(
                Ast::div(Ast::var("a"), Ast::var("b")),
                Ast::var("c"),
            )],
        );

        // Unary operators bind tighter than binary operators
        test_parser(
            "!a + b",
            &[Ast::plus(Ast::log_not(Ast::var("a")), Ast::var("b"))],
        );
        test_parser(
            "~a * b",
            &[Ast::mul(Ast::bit_not(Ast::var("a")), Ast::var("b"))],
        );

        // Parentheses group sub-expressions
        test_parser(
            "a * (b + c)",
            &[Ast::mul(
                Ast::var("a"),
                Ast::plus(Ast::var("b"), Ast::var("c")),
            )],
        );
        test_parser(
            "(a - b) % c",
            &[Ast::mod_(
                Ast::minus(Ast::var("a"), Ast::var("b")),
                Ast::var("c"),
            )],
        );
    }

    #[test]
    fn parse_assignment() {
        test_parser("a = b;", &[Ast::assign("a", Ast::var("b"))]);
        test_parser(
            "what = wow + !hey;",
            &[Ast::assign(
                "what",
                Ast::plus(Ast::var("wow"), Ast::log_not(Ast::var("hey"))),
            )],
        );
    }

    #[test]
    fn parse_print() {
        test_parser("print x;", &[Ast::print(Ast::var("x"))]);
        test_parser(
            "print true || b + x;",
            &[Ast::print(Ast::log_or(
                Ast::Boolean(true),
                Ast::plus(Ast::var("b"), Ast::var("x")),
            ))],
        );
    }

    #[test]
    fn parse_if() {
        test_parser(
            "if a { print b; }",
            &[Ast::if_cond(
                Ast::var("a"),
                vec![Ast::print(Ast::var("b"))],
                vec![],
            )],
        );

        test_parser(
            "if a || b { print b; } else { x }",
            &[Ast::if_cond(
                Ast::log_or(Ast::var("a"), Ast::var("b")),
                vec![Ast::print(Ast::var("b"))],
                vec![Ast::var("x")],
            )],
        );

        test_parser(
            "if a { a } else if b { b } else { c }",
            &[Ast::if_cond(
                Ast::var("a"),
                vec![Ast::var("a")],
                vec![Ast::if_cond(
                    Ast::var("b"),
                    vec![Ast::var("b")],
                    vec![Ast::var("c")],
                )],
            )],
        );

        test_parser(
            "if a { a } else if b { b } else if c { c }",
            &[Ast::if_cond(
                Ast::var("a"),
                vec![Ast::var("a")],
                vec![Ast::if_cond(
                    Ast::var("b"),
                    vec![Ast::var("b")],
                    vec![Ast::if_cond(Ast::var("c"), vec![Ast::var("c")], vec![])],
                )],
            )],
        );
    }

    #[test]
    fn parse_complex_expression() {
        test_parser(
            "a + { print a; c }",
            &[Ast::plus(
                Ast::var("a"),
                Ast::Block(vec![Ast::print(Ast::var("a")), Ast::var("c")]),
            )],
        );

        test_parser(
            "if a { b } * x + y",
            &[Ast::plus(
                Ast::mul(
                    Ast::if_cond(Ast::var("a"), vec![Ast::var("b")], vec![]),
                    Ast::var("x"),
                ),
                Ast::var("y"),
            )],
        );
    }

    #[test]
    fn parse_example2() {
        let input = r#"
a = 1;
b = 0;

if b == a {
    print 0;
}

if a > b {
    print 2; // Get some comments in here too, why not
}

// Lonely little line comment

/* Block comment even!
 * Look at this, multiple lines! */

print a + b;
"#;
        test_parser(
            input,
            &[
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
            ],
        );
    }
}
