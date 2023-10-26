# The HypeScript language reference

This document is a mostly-informal overview of the syntax and semantics of the HypeScript
language.

> Note: Although this document describes both abstract and concrete syntax, there is as
> yet no parser for HypeScript; only the abstract syntax is implemented.

## Notation

In definitions of lexical and syntactic constructs, the following notations are used:

| Notation          | Examples                           | Definition                      |
|-------------------|------------------------------------|---------------------------------|
| CAPITAL           | VAR\_NAME                          | A lexical token                 |
| _Italic_          | _Assignment_                       | A syntactic construct           |
| `string`          | `if`, `=`                          | Exact characters                |
| x<sup>?</sup>     | _ElseClause_<sup>?</sup>           | Optional item                   |
| x<sup>\*</sup>    | _ElseIfClause_<sup>\*</sup>        | 0 or more repetitions of x      |
| x<sup>\+</sup>    | _ElseIfClause_<sup>\*</sup>        | 1 or more repetitions of x      |
| \[ `-` ]          | [`0-9`]                            | Any single character in a range |
| \|                | _Assignment_ \| _Block_            | Either one or another item      |
| ( )               | (`else` `{` _Seq_ `}`)<sup>?</sup> | Item grouping                   |

## Data types

The HypeScript compiler does not perform any type checking. Nonetheless, there are the
following data types:

- Integers.
- Booleans.

At the binary level, integers are 64-bit unsigned integers. Booleans are also represented
by integers, but only the values 0 (false) and 1 (true) are valid.

A hypothetical type checker would forbid the use of integers as conditions for `if`
statements and the like. In the absence of a type checker, all nonzero values are regarded
as true for the purposes of condition checking, and only 0 is regarded as false.

Additionally, there is a conceptual unit type, which is never represented as a concrete
value during program execution and whose "values" will usually produce a runtime error if
used for variable assignment or the like. It is useful conceptually, to allow for uniform
treatment of statements and expressions in the language specification.

## Lexical structure

### Keywords

> KEYWORD: `if` \| `else` \| `print` \| `true` \| `false`

### Identifiers

> IDENT:\
> &nbsp;&nbsp; &nbsp;&nbsp; (ID\_START ID\_CONTINUE<sup>\*</sup>\
> &nbsp;&nbsp; \| `_` ID\_CONTINUE<sup>+</sup>)<sub>Except KEYWORDs<sub>

ID\_START and ID\_CONTINUE correspond to the Unicode character classes ID\_Start and
ID\_Continue.

### Literals

#### Integer literals

> DEC\_DIGIT: \[`0-9`]\
> \
> HEX\_DIGIT: \[`0-9`] \| \[`a-f`] \| \[`A-F`]\
> \
> INT\_LITERAL: DEC\_LITERAL \| HEX\_LITERAL\
> \
> DEC\_LITERAL: DEC\_DIGIT<sup>+</sup>\
> \
> HEX\_LITERAL: `0x` HEX\_DIGIT<sup>+</sup>

#### Boolean literals

> BOOL\_LITERAL: `true` \| `false`

#### Operators

> BINARY\_OPERATOR:\
> &nbsp;&nbsp; &nbsp;&nbsp; `+`\
> &nbsp;&nbsp; \| `-`\
> &nbsp;&nbsp; \| `*`\
> &nbsp;&nbsp; \| `/`\
> &nbsp;&nbsp; \| `%`\
> &nbsp;&nbsp; \| `>`\
> &nbsp;&nbsp; \| `<`\
> &nbsp;&nbsp; \| `>=`\
> &nbsp;&nbsp; \| `<=`\
> &nbsp;&nbsp; \| `==`\
> &nbsp;&nbsp; \| `!=`\
> &nbsp;&nbsp; \| `&`\
> &nbsp;&nbsp; \| `|`\
> &nbsp;&nbsp; \| `^`\
> &nbsp;&nbsp; \| `&&`\
> &nbsp;&nbsp; \| `||`\
> \
> UNARY\_OPERATOR: `~` \| `!`
 
## Abstract syntax

### Program structure

> _Program_: _Statement_<sup>+</sup>\
> \
> _Statement_: _AssignmentStatement_ \| _PrintStatement_ \| _Expression_

A HypeScript program consists of a sequence of statements. Statements can take the
following forms:

- Variable assignments, which bind values to variable names.
- Print statements, which emit a value to the output stream.
- Value expressions.

### Variable assignment

> _Assignment_: IDENT `=` _Expression_ `;`

A variable assignment statement evaluates an expression and binds its result to a
variable. Assignment implicitly declares variables that have not yet been declared, and
variables may not be used in expressions before being declared and assigned to in this
way.

> Note: It is syntactically possible to assign expressions of unit type, though this will
> almost always result in a runtime stack underflow error. A type checker for HypeScript
> would forbid this.

### Print statements

> _PrintStatement_: `print` _Expression_ `;`

A print statement evaluates an expression and emits its result to the output stream as an
ASCII decimal integer.

> Note: As with assignment statements, it is syntactically possible to print expressions
> of unit type, and this will usually result in a runtime error.

### Expressions

> _Expression_:\
> &nbsp;&nbsp; &nbsp;&nbsp; _LiteralExpression_\
> &nbsp;&nbsp; \| _VariableExpression_\
> &nbsp;&nbsp; \| _ArithOrBooleanExpression_\
> &nbsp;&nbsp; \| _BlockExpression_\
> &nbsp;&nbsp; \| _IfExpression_

An expression statement yields a value, which may be of unit type in the case of
_IfExpression_ and _BlockExpression_.

#### Literal expressions

> _LiteralExpression_: INT\_LITERAL \| BOOL\_LITERAL

A literal expression is simply a constant integer or boolean value.

#### Variable expressions

> _VariableExpression_: IDENT

A variable expression yields the current value of the named variable.

Variables must be in scope at the time of use; see the later section on scoping rules for
further details.

#### Arithmetic and boolean expressions

> _ArithOrBooleanExpression_:\
> &nbsp;&nbsp; &nbsp;&nbsp; (_Expression_ BINARY\_OPERATOR _Expression_)\
> &nbsp;&nbsp; (UNARY\_OPERATOR _Expression_)

The semantics of each operator can be summarized as follows:

| Operator | Operands             | Result       | Description                 |
|----------|----------------------|--------------|-----------------------------|
| `+`      | Integers             | Integer      | Addition<sup>1</sup>        |
| `-`      | Integers             | Integer      | Subtraction<sup>1</sup>     |
| `*`      | Integers             | Integer      | Multiplication<sup>1</sup>  |
| `/`      | Integers             | Integer      | Division<sup>2</sup>        |
| `%`      | Integers             | Integer      | Modulo/remainder            |
| `>`      | Integers             | Boolean      | Greater-than comparison     |
| `<`      | Integers             | Boolean      | Less-than comparison        |
| `>=`     | Integers             | Boolean      | Greater-or-equal comparison |
| `<=`     | Integers             | Boolean      | Less-or-equal comparison    |
| `==`     | Integers or booleans | Boolean      | Equality comparison         |
| `!=`     | Integers or booleans | Boolean      | Inequality comparison       |
| `&`      | Integers or booleans | Operand type | Bitwise AND                 |
| `\|`     | Integers or booleans | Operand type | Bitwise OR                  |
| `^`      | Integers or booleans | Operand type | Bitwise XOR                 |
| `&&`     | Booleans             | Boolean      | Logical AND<sup>3</sup>     |
| `\|\|`   | Booleans             | Boolean      | Logical OR<sup>3</sup>      |
| `~`      | Integer              | Integer      | Bitwise NOT<sup>4</sup>     |
| `!`      | Integer or boolean   | Boolean      | Logical NOT<sup>4</sup>     |

<sup>1</sup> All arithmetic silently wraps on overflow.

<sup>2</sup> Division truncates its result, and yields a runtime error when the divisor is
zero.

<sup>3</sup> Unlike in many languages, the logical connective operators are _not_
short-circuiting; they will fully evaluate both of their operands before computing their
final result. Additionally, `&&` and `||` are implemented using the same bytecode
instructions as `&` and `|`; thus, in the absence of a type checker, the logical and
bitwise AND and OR operators are in fact identical in their semantics: they both perform
_bitwise_ logical operations.

<sup>4</sup> Where the logical and bitwise binary operators behave identically, logical
and bitwise NOT have meaningfully differing semantics. In particular, bitwise NOT will
invert every bit in its operand, while logical NOT will yield 0 if its operand is nonzero,
and 1 otherwise. Two applications of logical NOT can therefore be used to convert an
integer value into its equivalent canonical boolean encoding.

#### Block expressions

> _BlockExpression_: `{` _Statement_<sup>\*</sup> `}`

Block expressions execute all statements contained within them, in sequence, and yield the
value of their final statement.

Additionally, block expressions introduce a new variable scope; see the later section on
scoping rules for further details.

Note: In practice, a block expression may yield _multiple_ values, but in a way that is
difficult and unintuitive to make use of. Any statement of non-unit type will conclude its
evaluation with its result value on the top of the stack; and so, if there are any
non-unit statements before the final statement of the block, they will be on the stack as
well. However, there is no way to access these extraneous values from a HypeScript program
except by (erroneously) providing a value of unit type as the value of an assignment, the
operand of a print statement, or the condition of an if clause. This will cause these
constructs to use the most recently evaluated extraneous value instead. A type checker
would forbid these extraneous values by ensuring that all statements in a block except for
the last are of unit type.

#### If expressions

> _IfExpression_:\
> &nbsp;&nbsp; `if` _Expression_ _BlockExpression_\
> &nbsp;&nbsp; (`else` `if` _Expression_ _BlockExpression_)<sup>\*</sup>\
> &nbsp;&nbsp; (`else` _BlockExpression_)<sup>?</sup>

An if expression consists of a single `if` clause, a chain of any number of following
`else if` clauses, and an optional concluding `else` clause.

Evaluation of an if expression proceeds as follows:

- The condition expression of the leading `if` clause is evaluated. If it is non-zero, the
  block expression of the `if` clause is evaluated, and the remainder of the expression is
  skipped.
- If the first condition expression yields 0 and there is at least one `else if` clause,
  the conditions for the `else if` clauses are evaluated, in sequence, until one yields a
  nonzero value. The first clause whose condition is nonzero is evaluated, and the
  remainder are skipped.
- If no condition in the chain evaluates to a nonzero value and there is an `else` clause,
  the `else` clause is evaluated.

Thus, at most one clause of an if expression is evaluated; the value of the expression is
the value yielded by this clause.

Note that it is possible for _no_ clauses to be evaluated, if there is no terminating
`else` clause; in this case, the expression yields a value of unit type. Note also that
different clauses may yield different types. Therefore, if expressions are in fact of
non-deterministic type. A type checker would ensure the following:

- Every clause in the expression must be of the same type.
- If there is no `else` clause, every clause in the expression must be of unit type.

## Variable scope

Scopes exist in a strict hierarchical tree structure. There is an implicit global scope,
and each block expression introduces a new scope that is enclosed by the scope in which
the block occurs.

The first assignment to a variable implicitly declares that variable, and each variable
belongs to the scope in which it was declared. Variables can only be used in expressions
if they belong to the current or an enclosing scope.

Additionally, a variable cannot be used in an expression before it is declared, even if it
is declared in the current or an enclosing scope. Variables are not considered to be
declared until after the assignment expression is evaluated; hence, variables cannot be
declared using expressions that contain themselves.

In summary, in order for a variable to be used in an expression:

- The variable must have been declared before the expression occurs;
- The variable must belong to the current or an enclosing scope.
