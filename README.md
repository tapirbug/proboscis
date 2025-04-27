# proboscis

This is a simple non-optimizing compiler for a tiny subset of common LISP.

It's written in Rust, that is, it is not self-hosting.

## Grammar
The grammar implemented by the top-down parser is something like this,
ignoring whitespace and comments, terminals in all caps:
```
program = list*
list = "(" elem* ")"
raw_list = "'" "(" elem* ")"
elem = list | raw_list | INT | FLOAT | STRING
```