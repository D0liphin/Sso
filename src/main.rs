#![feature(
    vec_push_within_capacity,
    allocator_api,
    alloc_layout_extra,
    slice_ptr_get,
    maybe_uninit_uninit_array_transpose,
    ptr_metadata,
)]

mod sso;
mod unsafe_field;

use sso::SsoString as String;

fn main() {
    let mut s = String::new();
    s.push_str("Hello, world!");
    println!("{:?}", s);
}
