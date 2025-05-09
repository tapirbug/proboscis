# proboscis

This is a simple non-optimizing compiler for a tiny subset of common LISP.

It's written in Rust, that is, it is not self-hosting.

## Runtime behavior
All data on the heap is in tagged unions called variants.

The data is set up with constant data first, then the heap, and then the stack.

The first piece of constant data is the nil list, which is always at address 0,
and thus contains only zero.
