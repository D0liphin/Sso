#![feature(
    vec_push_within_capacity,
    allocator_api,
    alloc_layout_extra,
    slice_ptr_get,
    maybe_uninit_uninit_array_transpose,
    ptr_metadata,
)]

mod sso;

type String = sso::SsoString;

fn main() {
    // let mut string = Str::from("Oliver Iliffe");

    // string.push_str("Hello, world!");
    // println!("{:?}", string);

    // string.push_str(" My name i");
    // println!("{:?}", string);

    // string.push_str("s Greg!");
    // println!("{:?}", string);

    let mut s = String::from("Hello, world,");
    assert!(s.is_short());
    s.push_str(" my name i");
    assert!(s.is_short());
    s.push_str("s");
    assert!(s.is_long());
    s.push_str(" George!");
    assert_eq!("Hello, world, my name is George!", &s);
}
