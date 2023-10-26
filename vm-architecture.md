# The HypeScript virtual machine and bytecode specification

## Architectural overview

The HypeScript VM is a stack machine; most instructions take their operands implicitly
from a FILO operand stack. All values on the stack are 64 bits.

In addition to the operand stack, programs have access to a random-access, resizable array
of variables. These variables are mutable 64-bit values indexed by their position in the
array, and their values can be freely copied to and from the stack.

All instructions consist of a one-byte opcode followed, in the case of inline literal
instructions, by a literal value of 1, 2, 4, or 8 bytes. All literals are loaded as 64-bit
values, but where possible literals may be encoded in smaller representations to reduce
code size; the VM will perform zero- or sign-extension as necessary.

The program and data memory spaces of the VM are fully separate, and programs have no
access to program memory, except in the indirect and limited way provided by the inline
literal instructions.

The VM supports three types natively: unsigned integers, signed two's complement integers,
and booleans. All stack and variable values can be freely reinterpreted as any of these
types, depending upon the particular instruction being executed.

The VM supports some primitive input and output capabilities; individual integers can be
read from an input stream, or written to an output stream.

> Design notes: Hypothetically, the VM might have three memory spaces: a global
> byte-addressed memory space for storing large or non-local data, e.g. strings or byte
> arrays; arrays of variables local to each call frame; and a global array of external
> variables, which could be bound to data outside of the VM to allow bytecode programs to
> interact with and manipulate the larger game engine. In this prototype, only the local
> variable arrays are implemented, and since function calls are not supported, there is in
> fact only one such array.

## Machine initialization and execution

The VM has the following state:

- The program being executed.
- A program counter register, containing the byte index of the next instruction to be
  executed.
- The operand stack.
- The local variable array.

The VM must be re-initialized for every execution of a program. When initialized, the
machine is in the following state:

- The program to be executed is loaded into program memory.
- The program counter is set to 0.
- Both the stack and the local variable arrays are empty.

The VM then begins execution in steps. On each step, the VM reads the opcode at the
program index specified by the program counter, reads any inline literal bytes required by
the opcode, and updates the machine state (stack, variables, program counter) and performs
input and output as required by the opcode. The program counter is then incremented to the
start of the next instruction.

The VM halts when either the program counter goes out of bounds of the program, or an
explicit halt instruction is executed. Opcodes that are not recognized are treated as
no-ops, and execution simply continues to the next byte in the program.

## Instruction listing

Notes:

- All instructions that pop or access stack values will halt the machine with a runtime
  error if the stack does not contain enough values to support their operation.
- All instructions that read or write a local variable will halt the machine with a
  runtime error if they attempt to access a variable index outside the current bounds of
  the variable array.

### Variable management instructions

- `varst` Store variable

  Pop an index N from the stack. Pop a value X from the stack. Store the value X in the
  local variable at index N.

- `varld` Load variable

  Pop an index N from the stack. Push the value of the local variable at index N onto the
  stack.

- `varres` Reserve variables

  Pop an unsigned integer N from the stack. Extend the local variable array by N slots.

  New variables are initialized to zero.

- `vardisc` Discard variables

  Pop an unsigned integer N from the stack. Shrink the local variable array by N slots.

  If N is larger than the current number of variables, no error is raised and the variable
  array will simply be cleared.

- `numvars` Query number of variables

  Push the current size of the local variable array onto the stack.

### Stack manipulation instructions

- `push8` `push16` `push32` `push64` Push literal

  Read an inline literal of 8, 16, 32, or 64 bits, depending on the variant, in big-endian
  byte order. Push the literal to the stack, zero-extending to 64 bits.

  This will halt the machine with a runtime error if there are insufficient bytes
  remaining in the program for the expected literal.

- `push8s` `push16s` `push32s` Push signed literal

  Read an inline literal of 8, 16, or 32 bits, depending on the variant, in big-endian
  byte order. Push the literal to the stack, sign-extending to 64 bits.

  This will halt the machine with a runtime error if there are insufficient bytes
  remaining in the program for the expected literal.

  Note that `push64s` is excluded from the instruction set, as it would be identical in
  operation to `push64`.

- `dup0` `dup1` `dup2` `dup3` Duplication stack slot

  Copy the stack value at index 0, 1, 2, or 3 in the stack, depending on the variant,
  indexing from the top of the stack. Push the copied value.

- `pop` Pop stack value

  Pop the top value from the stack.

- `swap` Swap top two stack values

  Swap the positions of the top two stack values.

### Arithmetic instructions

- `add` Addition

  Pop an integer B from the stack. Pop an integer A from the stack. Compute the sum A + B
  and push the result to the stack.

  This instruction silently wraps on overflow.

- `sub` Subtraction

  Pop an integer B from the stack. Pop an integer A from the stack. Compute the difference
  A - B and push the result to the stack.

  This instruction silently wraps on overflow.

- `mul` Multiplication

  Pop an integer B from the stack. Pop an integer A from the stack. Compute the product A
  × B and push the result to the stack.

  This instruction silently wraps on overflow.

- `div` `divs` Division

  Pop an integer B from the stack. Pop an integer A from the stack. Compute the quotient A
  / B, truncating towards zero.

  `div` regards its operands as unsigned; `divs` regards them as signed.

  This instruction will halt the machine with a runtime error if B is zero.

- `mod` Modulo

  Pop an unsigned integer B from the stack. Pop an unsigned integer A from the stack.
  Compute the remainder A mod B and push the result to the stack.

  This instruction will halt the machine with a runtime error if B is zero.

### Comparison instructions

- `gt` `gts` Greater than

  Pop an integer B. Pop an integer A. If A is greater than B, push 1; otherwise push 0.

  `gt` regards its operands as unsigned; `gts` regards them as signed.

- `lt` `lts` Less than

  Pop an integer B. Pop an integer A. If A is less than B, push 1; otherwise push 0.

  `lt` regards its operands as unsigned; `lts` regards them as signed.

- `ge` `ges` Greater or equal

  Pop an integer B. Pop an integer A. If A is greater than or equal to B, push 1;
  otherwise push 0.

  `ge` regards its operands as unsigned; `ges` regards them as signed.

- `le` `les` Less or equal

  Pop an integer B. Pop an integer A. If A is less than or equal to B, push 1; otherwise
  push 0.

  `le` regards its operands as unsigned; `les` regards them as signed.

- `eq` Equal

  Pop a value B. Pop a value A. If A is equal to B, push 1; otherwise push 0.

### Logical and bitwise instructions

- `and` `or` `xor` Bitwise binary operators

  Pop a value B. Pop a value A. Compute the bitwise AND, OR, or XOR of A and B, and push
  the result.

- `not` Logical negation

  Pop a value A. If A is zero, push 1; otherwise, push 0.

- `inv` Bitwise inversion

  Pop a value A. Compute the bitwise NOT of A and push the result.

### Control flow instructions

- `jump` Unconditional jump

  Pop a signed offset N. Add N to the program counter.

- `jcond` Conditional jump

  Pop a signed offset N. Pop a value C. If C is nonzero, add N to the program counter.

Note: the program counter is incremented by 1 after each jump instruction is executed.
Thus, the next instruction to be executed after jumping will be PC + 1 + N, where PC is
the program address of the jump instruction.

### Input and output instructions

- `read` `reads` Read value from input

  Parse an integer from the input stream, and push it to the stack.

  This assumes that the input stream is a UTF-8 text stream. It will skip leading
  whitespace, and attempt to parse a base-10 ASCII integer. `read` expects an unsigned,
  positive integer; `reads` will accept negative integers.

  These instructions will halt the machine with a runtime error for any of the following
  reasons:
  - Reading from the input stream fails (e.g. due to early end of stream, or other host
    platform exception).
  - The received characters cannot be parsed as an integer.
  - The parsed integer overflows the 64-bit stack slot.

- `print` `prints` Print value to output

  Pop an integer from the stack, and print it to the output stream followed by a newline.

  This will format the value as a base-10 ASCII integer. `print` will format it as an
  unsigned positive integer; `prints` will format it as a signed integer.

  These instructions will halt the machine with a runtime error if writing to the output
  stream fails.

### Miscellaneous instructions

- `halt` Halt execution

  Immediately halt program execution.

## Opcode listing

| Opcode (hexadecimal) | Instruction |
|----------------------|-------------|
| 0x18                 | varst       |
| 0x1a                 | varld       |
| 0x1c                 | varres      |
| 0x1d                 | vardisc     |
| 0x1e                 | numvars     |
| 0x28                 | push8       |
| 0x29                 | push8s      |
| 0x2a                 | push16      |
| 0x2b                 | push16s     |
| 0x2c                 | push32      |
| 0x2d                 | push32s     |
| 0x2e                 | push64      |
| 0x30                 | dup0        |
| 0x31                 | dup1        |
| 0x32                 | dup2        |
| 0x33                 | dup3        |
| 0x34                 | pop         |
| 0x35                 | swap        |
| 0x38                 | add         |
| 0x39                 | sub         |
| 0x3a                 | mul         |
| 0x3b                 | mod         |
| 0x3c                 | div         |
| 0x3d                 | divs        |
| 0x50                 | gt          |
| 0x51                 | gts         |
| 0x52                 | lt          |
| 0x53                 | lts         |
| 0x54                 | ge          |
| 0x55                 | ges         |
| 0x56                 | le          |
| 0x57                 | les         |
| 0x58                 | eq          |
| 0x59                 | and         |
| 0x5a                 | or          |
| 0x5b                 | xor         |
| 0x5c                 | not         |
| 0x5d                 | inv         |
| 0x60                 | jump        |
| 0x61                 | jcond       |
| 0xfa                 | read        |
| 0xfb                 | reads       |
| 0xfc                 | print       |
| 0xfd                 | prints      |
| 0xff                 | halt        |

> Design notes: Opcodes are generally allocated so that broad categories of instructions
> begin on a multiple of 8, with gaps in the allocation allowed in order to accomodate
> this. Additionally, where there are instruction variants that differ only in the
> signedness of their operands, their opcodes are identical except that the unsigned
> version has the low bit cleared, and the signed version has the low bit set.

> There are several strange gaps in the allocated opcodes; mostly these are for
> instructions that were dreamed up but not implemented. Of particular note, opcodes were
> allocated for managing the unimplemented other memory spaces discussed in architectural
> overview. Opcodes 0x00 through 0x0e were allocated towards management of main
> byte-addressed memory, and the one-opcode gaps between the local variable management
> instructions were allocated towards the similar instructions for external variable
> management.
