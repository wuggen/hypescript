# The HypeScript language reference

This document is a mostly-informal overview of the syntax and semantics of the HypeScript
language.

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

## Type system

HypeScript supports the following primitive types:

- Unsigned integers;
- Booleans;
- Unit.

At the binary level, integers are 64-bit unsigned integers. Booleans are also represented
by integers, but only the values 0 (false) and 1 (true) are valid.

Values of Unit type are never represented at runtime, and cannot be assigned to variables;
nonetheless, the type is useful in the definition of HypeScript's type system, to allow
for uniform treatment of statements and expressions that may not yield values.

HypeScript is strictly typed, and no implicit or explicit coercion between types is
permitted.

## Lexical structure

### Keywords

> KEYWORD: `if` \| `else` \| `print` \| `true` \| `false`

### Identifiers

> IDENT:\
> &nbsp;&nbsp; &nbsp;&nbsp; (ID\_START ID\_CONTINUE<sup>\*</sup>
> \
> ID\_START: `_` \| [`a-z`] \| [`A-Z`] \
> \
> ID\_CONTINUE: ID\_START \| [`0-9`]

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
> &nbsp;&nbsp; \| `\|`\
> &nbsp;&nbsp; \| `^`\
> &nbsp;&nbsp; \| `&&`\
> &nbsp;&nbsp; \| `\|\|`\
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

The type of a variable is determined upon declaration, and is the same as the type of the
expression assigned to it. While all variables' values are mutable, their types are not; a
variable cannot be later reassigned to a value of a different type. Expressions of Unit
type cannot be used to declare a variable.

Assignment statements are of Unit type.

### Print statements

> _PrintStatement_: `print` _Expression_ `;`

A print statement evaluates an expression and emits its result to the output stream as an
ASCII decimal integer.

The printed value must be a well-typed Integer or Boolean expression. For Booleans, print
statements will emit a 0 for false, and a 1 for true.

Print statements are of Unit type.

### Expressions

> _Expression_:\
> &nbsp;&nbsp; &nbsp;&nbsp; _LiteralExpression_\
> &nbsp;&nbsp; \| _VariableExpression_\
> &nbsp;&nbsp; \| _ArithOrBooleanExpression_\
> &nbsp;&nbsp; \| _BlockExpression_\
> &nbsp;&nbsp; \| _IfExpression_

An expression statement yields a value, which may be of any type, including Unit in the
case of _IfExpression_ and _BlockExpression_.

#### Literal expressions

> _LiteralExpression_: INT\_LITERAL \| BOOL\_LITERAL

A literal expression is simply a constant integer or boolean value, and takes on the
corresponding type.

#### Variable expressions

> _VariableExpression_: IDENT

A variable expression yields the current value of the named variable, and has the type
with which that variable was declared.

Variables must be in scope at the time of use; see the later section on scoping rules for
further details.

#### Arithmetic and boolean expressions

> _ArithOrBooleanExpression_:\
> &nbsp;&nbsp; &nbsp;&nbsp; (_Expression_ BINARY\_OPERATOR _Expression_)\
> &nbsp;&nbsp; \| (UNARY\_OPERATOR _Expression_)

The semantics of each operator can be summarized as follows:

| Operator | Operand type(s)      | Result type  | Description                  |
|----------|----------------------|--------------|------------------------------|
| `+`      | Integers             | Integer      | Addition<sup>1</sup>         |
| `-`      | Integers             | Integer      | Subtraction<sup>1</sup>      |
| `*`      | Integers             | Integer      | Multiplication<sup>1</sup>   |
| `/`      | Integers             | Integer      | Division<sup>2</sup>         |
| `%`      | Integers             | Integer      | Modulo/remainder<sup>3</sup> |
| `>`      | Integers             | Boolean      | Greater-than comparison      |
| `<`      | Integers             | Boolean      | Less-than comparison         |
| `>=`     | Integers             | Boolean      | Greater-or-equal comparison  |
| `<=`     | Integers             | Boolean      | Less-or-equal comparison     |
| `==`     | Integers             | Boolean      | Equality comparison          |
| `!=`     | Integers             | Boolean      | Inequality comparison        |
| `&`      | Integers             | Integer      | Bitwise AND                  |
| `\|`     | Integers             | Integer      | Bitwise OR                   |
| `^`      | Integers             | Integer      | Bitwise XOR                  |
| `&&`     | Booleans             | Boolean      | Logical AND<sup>3</sup>      |
| `\|\|`   | Booleans             | Boolean      | Logical OR<sup>3</sup>       |
| `~`      | Integer              | Integer      | Bitwise NOT                  |
| `!`      | Boolean              | Boolean      | Logical NOT                  |

<sup>1</sup> All arithmetic silently wraps on overflow.

<sup>2</sup> Division truncates its result, and yields a runtime error when the divisor is
zero.

<sup>3</sup> Modulo yields a runtime error when the modulus is zero.

<sup>3</sup> Unlike in many languages, the logical connective operators are _not_
short-circuiting; they will fully evaluate both of their operands before computing their
final result.

Operators have the following binding levels, from strongest to weakest:

- Unary operators: `~` and `!`.
- Multiplication and devision operators: `*`, `/`, and `%`.
- Bitwise integer operators: `&`, `|`, and `^`.
- Addition and subtraction operators: `+` and `-`.
- Integer comparison operators: `>`, `>=`, `<`, `<=`, `==`, and `!=`.
- Logical AND: `&&`.
- Logical OR: `||`.

Within each binding level, all binary operators are left-associative. Parentheses may be
used to group sub-expressions. Literals, variables, `if` expressions, and block
expressions are parsed as atomic sub-expressions.

#### Block expressions

> _BlockExpression_: `{` _Statement_<sup>\*</sup> `}`

Block expressions execute all statements contained within them, in sequence, and yield the
value of their final statement. All statements in a block must be of Unit type, except for
possibly the final one; the block as a whole takes on the type of its final statement. And
empty block is of Unit type.

Additionally, block expressions introduce a new variable scope; see the later section on
scoping rules for further details.

#### If expressions

> _IfExpression_:\
> &nbsp;&nbsp; `if` _Expression_ _BlockExpression_\
> &nbsp;&nbsp; (`else` `if` _Expression_ _BlockExpression_)<sup>\*</sup>\
> &nbsp;&nbsp; (`else` _BlockExpression_)<sup>?</sup>

An `if` expression consists of a single `if` clause, a chain of any number of following
`else if` clauses, and an optional concluding `else` clause.

Evaluation of an `if` expression proceeds as follows:

- The condition expression of the leading `if` clause is evaluated. If it is non-zero, the
  block expression of the `if` clause is evaluated, and the remainder of the expression is
  skipped.
- If the first condition expression yields 0 and there is at least one `else if` clause,
  the conditions for the `else if` clauses are evaluated, in sequence, until one yields a
  nonzero value. The first clause whose condition is nonzero is evaluated, and the
  remainder are skipped.
- If no condition in the chain evaluates to a nonzero value and there is an `else` clause,
  the `else` clause is evaluated.

Thus, at most one clause of an `if` expression is evaluated; the value of the expression
is the value yielded by this clause.

If there is no concluding `else` clause, all clauses' block expressions must be of Unit
type, and the expression as a whole has Unit type. If there is a concluding `else`
statement, all clauses' block expressions must have the same type, and the expression as a
whole has that type.

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
