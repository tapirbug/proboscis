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

## Runtime behavior
All data on the heap is in tagged unions called variants.

The data is set up with constant data first, then the heap, and then the stack.

The first piece of constant data is the nil list, which is always at address 0,
and thus contains only zero.
