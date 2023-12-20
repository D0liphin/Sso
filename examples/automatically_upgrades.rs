use sso::String;

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
fn main() {
    let mut s = String::from("Hello, world!");
    assert!(s.is_short());
    assert!(!s.is_long());
    assert_eq!(&s, "Hello, world!");

    s += " My name is Gregory :)";
    assert!(s.is_long());
    assert!(!s.is_short());
    assert_eq!(&s, "Hello, world! My name is Gregory :)");
}

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
fn main() {
    panic!("{}", concat!(
        "this example cannot run, because it relies on small-string optimisation, which is",
        " not implemented on this endian + pointer_width"
    ))
}