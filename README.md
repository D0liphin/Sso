# `SsoString` in Rust

Small string optimisation is done only for strings of length 23 or less. 

This crate defines a single non-conditional export `String` which is either `sso::SsoString`, or 
`std::string::String` depending on your architecture.

Small string optimisation is only available on `#[cfg(all(target_endian = "little", target_pointer_width = "64"))]`.

# Can I use this?

This is an imaginary conversation I am having with a person who will never exist, but I would recommend strongly that 
you do not use this, unless perhaps you can guarantee that the exported type is actually `std::string::String` hehe.

But seriously, although tested a little, it's not rigorously safe yet. Once I add debug assertions about unsafe 
preconditions, I'll be more confident that it is safe to use.

For now, everything *appears* to be safe, but nothing is as it seems in the land of `unsafe`! 

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

It's also why I use `len: UnsafeWrite<usize, 0>`. So that I cannot accidentally set the length to an invalid value 
without using `unsafe`, which reminds me to clear the safety contract i might be violating.

And why I can't `impl Drop`, because otherwise a semantically simultaneous write (not realy true, but it's good 
enough) is impossible for `capacity` and `buf`. E.g. this code would become impossible (I need to write both 
`capacity` and `buf` 'at the same time', so that `LongString` is never invalid.

```rs
/// free the buffer of this string, setting the `len` and `capacity` to `0`
pub fn free(&mut self) {
    let capacity = self.capacity();
    *self = unsafe {
        Self {
            // SAFETY: 0 always satisfies len's invaraints
            len: UnsafeWrite::new(0),
            // SAFETY: the buffer is dangling and the capacity is 0, which is a valid
            // state for LongString
            capacity: UnsafeWrite::new(0),
            buf: UnsafeWrite::new(
                self.buf
                    .own()
                    // SAFETY: capacity is the exact size of the buffer
                    .dealloc(capacity)
                    .expect("should be the exact capacity"),
            ),
        }
    };
}
```
