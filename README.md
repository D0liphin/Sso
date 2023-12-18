# `SsoString` in Rust

Small string optimisation is done only for strings of length 23 or less. 

This crate defines a single non-conditional export `String` which is either `sso::SsoString`, or 
`std::string::String` depending on your architecture.

Smal string optimisation is only available on `#[cfg(all(target_endian = "little", target_pointer_width = "64"))]`.

## Usage

```rs
use sso::String;

let mut s = String::new();
s += "Hello, world!";
assert_eq!(&s, "Hello, world!");
```

The goal is to have this completely replace `String`.

## Why is your code weird?

A longer explanation to come. The idea is to uphold the invariants of the struct **at all times**, instead of just 
when they might actually cause UB. Basically, trying to make `unsafe` code really, really simple to prove safety.

That's why all my code has `# Safety` contracts and `SAFETY:` contract clearances at every `unsafe` call-site 
(I think).

That's why I use `len: UnsafeWrite<usize, 0>`. So that I cannot accidentally set the length to an invalid value 
without using `unsafe`, which reminds me to clear the safety contract i might be violating.
