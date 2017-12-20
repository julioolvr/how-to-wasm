# How to WASM

The purpose of this repo is to organize myself on my path to learn how to compile Rust code
to WASM. Specifically, I want to compile [`rlox`](http://github.com/julioolvr/rlox), my Lox
interpreter based on the Java interpreter described in
[Crafting Interpreters](http://github.com/julioolvr/rlox).

Given that there might be many details that I'll have to review that are very specific to my
rlox project, I'll consider that I'm done with this repo once I'm able to compile a Rust program
that exports a function which receives a string and returns another string. Once I have that,
I just have to implement that interface for rlox - to receive a string with all the source code
and return a string with all the standard output. This might not be exactly like that, since
after some preliminary reading it seems that the way WASM and JavaScript code is through pointers
to memory, but ideally I'd like to end with some JS API on top of the WASM module that makes that
transparent.