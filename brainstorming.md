# Nillion tech challenge brainstorming

## Architectural overview

- Stack machine: instruction operands are given and results returned on an
  arbitrarily-sized stack of 64-bit values.
- Instructions consist of a one-byte opcode, followed by an optional inline argument of 1,
  2, 4, or 8 bytes, depending on the opcode.
- Execution of instructions proceeds sequentially through the program code, until either
  the last instruction or a `halt` instruction is executed. Unrecognized opcodes are
  ignored and act as no-ops.
- Control flow can be manipulated by the `j` and `jcond` instructions. These modify the
  program counter either unconditionally (`j`) or conditionally (`jcond`) by popping a
  value from the stack and adding it to the current value of the program counter.
- The virtual machine has three memory spaces:
  - Main memory, a linear byte-addressed memory space. Can be expanded or contracted by
    dedicated instructions.
  - External variables: an index-addressed array of mutable 64-bit values. Of fized size,
    determined at VM instantiation.
  - Local variables: an index-addressed array of mutable 64-bit values. Can be expanded or
    contracted by dedicated instructions. Each frame on the call stack has its own local
    variables array which cannot be accessed by any other frame.

## Instruction listing

### Main memory

- **`memstN`**  (`N` = 8, 16, 32, 64) Store value in memory

  **Pop:** Address A, value X

  **Effects:** Store the value X in main memory at address A.

  **Variants:** The 8, 16, and 32 bit variants truncate the high-order bytes of X, storing
  only the low-order 1, 2, or 4 bytes.

  **Exceptions:** Address A is outside the bounds of main memory.

- **`memldN`**  (`N` = 8, 16, 32, 64) Load value from memory

  **Pop:** Address A

  **Push:** The value in main memory at address A.

  **Variants:** The 8, 16, and 32 bit variants read only 1, 2, or 4 bytes from main memory,
  and copy them to the low-order bytes of the stack value. The remaining high-order bytes
  are cleared (set to zero).

  **Exceptions:** Address A is outside the bounds of main memory.

- **`memldNs`**  (`N` = 8, 16, 32) Load signed value from memory

  **Pop:** Address A

  **Push:** The value in main memory at address A, regarded as a signed integer.

  **Variants:** Reads only 1, 2, or 4 bytes from main memory, storing them in the low-order
  bytes of the stack value, with sign extension.

  **Exceptions:** Address A is outside the bounds of main memory.

  **Notes:** `memld64s` is omitted from the instruction set, as it would be identical in
  operation to `memld64`.

- **`memres`** Reserve additional memory space

  **Pop:** Integer N

  **Effects:** Extend main memory by N bytes, initializing new bytes to zero.

  **Exceptions:** The host platform is unable to reserve sufficient memory space.

- **`memdisc`** Discard memory space

  **Pop:** Integer N

  **Effects:** Shrink main memory by N bytes, discarding higher-addressed bytes.

  **Exceptions:** Main memory is smaller than N bytes.

- **`memsize`** Query memory size

  **Push:** The current size of main memory, in bytes.

### Local and external variables

- **`varst` `extst`** Store variable

  **Pop:** Index N, value X

  **Effects:** Store X in the local (resp. external) variable at index N.

  **Exceptions:** The index N is outside the bounds of the current local (resp. external)
  variable array.

- **`varld` `extld`** Load variable

  **Pop:** Index N

  **Push:** The value of the local (resp. external) variable at index N.

  **Exceptions:** The index N is outside the bounds of the current local (resp. external)
  variable array.

- **`varres`** Reserve additional variables

  **Pop:** Integer N

  **Effects:** Extend the current local variable array by N slots, initializing new slots
  to zero.

  **Exceptions:** The host platform is unable to reserve sufficient memory space.

- **`vardisc`** Discard variables

  **Pop:** Integer N

  **Effects:** Shrink the current local variable array by N slots, discarding higher-index
  values.

  **Exceptions:** The local variable array is shorter than N slots.

- **`numvars` `numext`** Query number of variables

  **Push:** The current number of local (resp. external) variable slots.

### Stack manipulation

- **`pushN`**  (`N` = 8, 16, 32, 64) Push literal

  **Inline arguments:** Value N

  **Push:** N

  **Variants:** The 8, 16, 32, and 64 bit variants take the following 1, 2, 4, or 8 bytes
  as their argument, respectively.

  **Exceptions:** There are insufficient bytes remaining in the program.

- **`pushNs`**  (`N` = 8, 16, 32) Push signed literal

  **Inline arguments:** Signed integer N

  **Push:** N, with sign extension.

  **Variants:** The 8, 16, and 32 bit variants take the following 1, 2, or 4 bytes as
  their argument, respectively.

  **Exceptions:** There are insufficient bytes remaining in the program.

  **Notes:** `push64s` is omitted from the instruction set, as it would be identical in
  operation to `push64`.

- **`dupN`**  (`N` = 0, 1, 2, 3) Duplicate stack slot

  **Push:** The value of the N'th stack slot from the top.

- **`pop`** Pop stack value

  **Pop:** Value X

- **`swap`** Swap top two stack values

  **Pop:** Values A, B

  **Push:** A, B

### Integer arithmetic

- **`add`** Integer addition

  **Pop:** Integers A, B

  **Push:** A + B

- **`sub`** Integer subtraction

  **Pop:** Integers A, B

  **Push:** A - B

- **`mul` `muls`** Integer multiplication

  **Pop:** Integers A, B

  **Push:** A \* B

  **Variants:** `mul` regards its operands as unsigned. `muls` regards them as signed.

- **`div` `divs`** Integer division

  **Pop:** Integers A, B

  **Push:** A / B (truncating towards zero)

  **Variants:** `div` regards its operands as unsigned. `divs` regards them as signed.

  **Exceptions:** B is zero.

- **`mod` `mods`** Integer modulo

  **Pop:** Integers A, B

  **Push:** A mod B

  **Variants:** `mod` regards its operands as unsigned. `mods` regards them as signed.

  **Exceptions:** B is zero.

### Integer comparison

- **`gt` `gts`** Greater-than comparison

  **Pop:** Integers A, B

  **Push:** 1 if A < B, 0 otherwise

  **Variants:** `gt` regards its operands as unsigned. `gts` regards them as signed.

- **`lt` `lts`** Less-than comparison

  **Pop:** Integers A, B

  **Push:** 1 if A > B, 0 otherwise

  **Variants:** `lt` regards its operands as unsigned. `lts` regards them as signed.

- **`ge` `ges`** Greater-or-equal comparison

  **Pop:** Integers A, B

  **Push:** 1 if A >= B, 0 otherwise

  **Variants:** `ge` regards its operands as unsigned. `ges` regards them as signed.

- **`le` `les`** Less-or-equal comparison

  **Pop:** Integers A, B

  **Push:** 1 if A <= B, 0 otherwise

  **Variants:** `le` regards its operands as unsigned. `les` regards them as signed.

- **`eq`** Equality comparison

  **Pop:** Values A, B

  **Push:** 1 if A is bitwise equal to B, 0 otherwise.

### Logical and bitwise operations

- **`and`** Bitwise AND

  **Pop:** Values A, B

  **Push:** The bitwise AND of A and B.

- **`or`** Bitwise OR

  **Pop:** Values A, B

  **Push:** The bitwise OR of A and B.

- **`xor`** Bitwise XOR

  **Pop:** Values A, B

  **Push:** The bitwise XOR of A and B.

- **`not`** Logical NOT

  **Pop:** Value X

  **Push:** 1 if X is 0, 0 otherwise.

- **`inv`** Bitwise NOT

  **Pop:** Value X

  **Push:** The bitwise NOT of X.

### Control flow

- **`jump`** Unconditional jump

  **Pop:** Signed offset N

  **Effects:** Add N to the program counter.

  **Notes:** The program counter is incremented by 1 after the execution of this
  instruction, as normal. Therefore, the next executed instruction will be the instruction
  at byte offset N from the instruction _following_ the jump instruction.

- **`jcond`** Conditional jump

  **Pop:** Signed offset N, Boolean C

  **Effects:** If C is nonzero, add N to the program counter. Otherwise, do nothing.

  **Notes:** The program counter is incremented by 1 after the execution of this
  instruction, as normal. Therefore, if the jump is taken, the next executed instruction
  will be the instruction at byte offset N from the instruction _following_ the jump
  instruction.

### Miscellaneous

- **`read` `reads`** Read value from input

  **Push:** An unsigned integer read from the input stream.

  **Variants:** `read` attempts to parse an unsigned integer. `reads` attempts to parse a
  signed integer.

  **Exceptions:**
  - Reading from the stream fails (e.g. due to end of stream, or other exception).
  - The received characters cannot be parsed as an integer.
  - The parsed integer overflows the 64-bit stack slot.

  **Notes:** Input proceeds as follows:
  - Bytes are discarded from the input stream until the first non-whitespace character.
  - Bytes are read until the next whitespace character.
  - The bytes read are parsed as a base-10 ASCII integer.

- **`print` `prints`** Print value to output

  **Pop:** Integer N

  **Effects:** Write N to the output stream as a base-10 ASCII integer, followed by a
  newline.

  **Variants:** `print` regards N as unsigned. `prints` regards N as signed.

  **Exceptions:** Writing to the stream fails.

- **`halt`** Halt execution

  **Effects:** Immediately halts execution of the program.

## Opcode listing

| Opcode (hexadecimal) | Instruction |
|----------------------|-------------|
| 0x00                 | memst8      |
| 0x01                 | memst16     |
| 0x02                 | memst32     |
| 0x03                 | memst64     |
| 0x04                 | memres      |
| 0x05                 | memdisc     |
| 0x06                 | memsize     |
| 0x07                 | [reserved]  |
| 0x08                 | memld8      |
| 0x09                 | memld8s     |
| 0x0a                 | memld16     |
| 0x0b                 | memld16s    |
| 0x0c                 | memld32     |
| 0x0d                 | memld32s    |
| 0x0e                 | memld64     |
| 0x0f:0x17            | [reserved]  |
| 0x18                 | varst       |
| 0x19                 | extst       |
| 0x1a                 | varld       |
| 0x1b                 | extld       |
| 0x1c                 | varres      |
| 0x1d                 | vardisc     |
| 0x1e                 | numvars     |
| 0x1f                 | numext      |
| 0x20:0x27            | [reserved]  |
| 0x28                 | push8       |
| 0x29                 | push8s      |
| 0x2a                 | push16      |
| 0x2b                 | push16s     |
| 0x2c                 | push32      |
| 0x2d                 | push32s     |
| 0x2e                 | push64      |
| 0x2f                 | [reserved]  |
| 0x30                 | dup0        |
| 0x31                 | dup1        |
| 0x32                 | dup2        |
| 0x33                 | dup3        |
| 0x34                 | pop         |
| 0x35                 | swap        |
| 0x36:0x37            | [reserved]  |
| 0x38                 | add         |
| 0x39                 | sub         |
| 0x3a                 | mul         |
| 0x3b                 | muls        |
| 0x3c                 | div         |
| 0x3d                 | divs        |
| 0x3e                 | mod         |
| 0x3f                 | mods        |
| 0x40:0x4f            | [reserved]  |
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
| 0x5e:0x5f            | [reserved]  |
| 0x60                 | jump        |
| 0x61                 | jcond       |
| 0x62:0xf9            | [reserved]  |
| 0xfa                 | read        |
| 0xfb                 | reads       |
| 0xfc                 | print       |
| 0xfd                 | prints      |
| 0xfe                 | [reserved]  |
| 0xff                 | halt        |
