# C Compiler

A minimal compiler for a subset of C written in Rust.

## Progress (more for my own reference than anything)
- [x] Compiler driver (gcc preprocessor, assembler, and linker)
- [x] Unary operators
- [x] Binary operators
- [x] Logical and relational operators
- [x] Local variables
- [x] If statements and conditional expressions
- [x] Compound statements / scopes
- [x] Loops
- [ ] Functions
- [ ] ...

## Acknowledgements

I'm building this compiler in an incremental fashion as presented in the book [Writing a C Compiler](https://nostarch.com/writing-c-compiler) by Nora Sandler, as well as [the paper by Abdulaziz Ghuloum on Scheme](http://scheme2006.cs.uchicago.edu/11-ghuloum.pdf) which inspired it.
Additionally, I also frequently reference implementation details from [wrecc](https://github.com/PhilippRados/wrecc), which is also a C compiler written in Rust.