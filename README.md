# HypeScript, a DSL for the Hypothetical Game Engine

This repository is organized as follows:

- [`hypescript-lang`](hypescript-lang) Implementation of the HypeScript language. Contains
  AST definitions and code generation routines.
- [`hypescript-vm`](hypescript-vm) Implementation of the HypeScript virtual machine,
  allowing for configurable code execution.
- [`hypescript_bytecode`](hypescript_bytecode) A library for encoding, decoding, and
  manipulating HypeScript VM bytecode.
- [`hype`](hype) A standalone command-line interpreter for the HypeScript VM.
- [`hypescript-util`](hypescript-util) A (very) small miscellaneous utility library used
  by several other crates in this repository.

To build all crates, a simple `cargo build` should suffice.

Documentation on the VM architecture and semantics can be found in
[vm-architecture.md](vm-architecture.md). Brief usage notes on the `hype` tool can be
obtained by passing it the `--help` flag.
