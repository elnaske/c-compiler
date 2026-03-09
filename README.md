# C Compiler

A minimal compiler for a subset of C written in Rust.

## Progress (more for my own reference than anything)
- [x] Compiler driver (gcc preprocessor, assembler, and linker)
- [x] Minimal compiler that can return constants
  - [x] Lexer
  - [x] Parser
  - [x] Assembly generation
- [ ] Unary operators
- [ ] Binary operators
- [ ] Logical and relational operators
- [ ] Local variables
- [ ] If statements and conditional expressions
- [ ] ...

## Acknowledgements

I'm building this compiler in an incremental fashion as presented in the book [Writing a C Compiler](https://nostarch.com/writing-c-compiler) by Nora Sandler, as well as [the paper by Abdulaziz Ghuloum on Scheme](http://scheme2006.cs.uchicago.edu/11-ghuloum.pdf) which inspired it.
Additionally, I also frequently reference implementation details from [wrecc](https://github.com/PhilippRados/wrecc), which is also a C compiler written in Rust.