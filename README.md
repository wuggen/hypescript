# HypeScript, a DSL for the Hypothetical Game Engine

This repository is organized as follows:

- [`hypescript-lang`](hypescript-lang) Implementation of the HypeScript language. Contains
  AST definitions and code generation routines.
- [`hypescript-vm`](hypescript-vm) Implementation of the HypeScript virtual machine,
  allowing for configurable code execution.
- [`hypescript-bytecode`](hypescript-bytecode) A library for encoding, decoding, and
  manipulating HypeScript VM bytecode.
- [`hypec`](hypec) A standalone command-line compiler for the HypeScript language.
- [`hype`](hype) A standalone command-line interpreter for HypeScript bytecode programs.
- [`hypescript-util`](hypescript-util) A (very) small miscellaneous utility library used
  by several other crates in this repository.

To build all crates, a simple `cargo build` should suffice.

Documentation on the VM architecture and semantics can be found in
[vm-architecture.md](vm-architecture.md), and documentation on the language syntax and
semantics can be found in [language-overview.md](language-overview.md) Brief usage notes
on the `hype` and `hypec` tools can be obtained by passing them the `--help` flag.
