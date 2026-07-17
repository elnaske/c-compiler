# tricc

tricc (short for toy, Rust-implemented C compiler) is a small compiler for a subset of C written in Rust.
I originally started writing it as an overly ambitious final project for a data structures and algorithms class.
It implements most of the core logic of C and compiles to x86-64 assembly, using the gcc preprocessor, assembler, and linker for the remainder of the toolchain, with no other external dependencies.

## Features
In its current iteration, the compiler supports the following:
- Arithmetic, logical, and relational operators
- Local variables
- If statements and conditional expressions
- Scopes and lifetimes
- Loops
- Function calls and linking with library functions

Currently, the only supported type is `int`.
Other notably absent features include pointers, bitwise operators, and switch statements.
I've put development of these on hold to focus on other projects.

## Acknowledgements

- [Writing a C Compiler](https://nostarch.com/writing-c-compiler): Excellenct, easy-to-pick-up resource for anyone interested in compilers. The book also comes with a full [test suite](https://github.com/nlsandler/writing-a-c-compiler-tests), which I made heavy use of and was my main way of testing during development (hence the rather barren `tests/` directory in this repo).
- [wrecc](https://github.com/PhilippRados/wrecc): Another C compiler written in Rust. I found it to be a great reference for implementation details, especially since this was my first time writing a compiler as well as one of my first major Rust projects.
- [Claude's C compiler](https://github.com/anthropics/claudes-c-compiler) for inspiring me to do this project through the realization that if Anthropic can slop together a C compiler for $20k, it can't be hard enough for me to write one myself for free. Also shoutouts to [this](https://github.com/anthropics/claudes-c-compiler/issues/1) legendary first issue.
