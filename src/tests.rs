use std::{
    borrow::Cow,
    mem::{self, ManuallyDrop},
    ptr::{self, NonNull},
};

use crate::{RawBuf, SsoStr, TaggedSsoString64Mut};

type StdString = std::string::String;
type String = super::SsoString;
type ShortString = super::ShortString64;
type LongString = super::LongString;

fn assert_aligned<T>(ptr: *const T) {
    assert_eq!(ptr.align_offset(mem::align_of::<T>()), 0)
}

fn assert_non_null<T>(ptr: *const T) {
    assert_ne!(ptr, ptr::null())
}

#[test]
fn raw_buf_clones_correctly() {
    let (buf, ..) = RawBuf::<i32>::new(16);
    assert_eq!(buf.data, buf.clone().data);
}

#[test]
fn raw_buf_is_aligned_and_non_null() {
    let (buf, ..) = RawBuf::<i32>::new(16);
    assert_aligned(buf.as_ptr());
    assert_non_null(buf.as_ptr());
}

#[test]
fn raw_buf_capacity_is_correct() {
    fn assert_raw_buf_capacity_is_correct<T>() {
        let (_, byte_count) = RawBuf::<i32>::new(16);
        assert!(byte_count >= 16 * mem::size_of::<i32>());

        let (buf, byte_count) = RawBuf::<i32>::new(0);
        assert_eq!(buf.data, NonNull::<i32>::dangling());
        assert_eq!(byte_count, 0);
    }
    assert_raw_buf_capacity_is_correct::<i32>();
    assert_raw_buf_capacity_is_correct::<u8>();
}

#[test]
fn test_sso_string_upgrades() {
    let mut s = String::from("Hello, world,");
    assert!(s.is_short());
    s += " my name i";
    assert!(s.is_short());
    assert_eq!(s.len(), ShortString::MAX_CAPACITY);
    s += "s";
    assert!(s.is_long());
    assert_eq!(&s, "Hello, world, my name is");
    s += " George!";
    assert_eq!("Hello, world, my name is George!", &s);
}

#[test]
fn short_string_64_fills_to_max_capacity() {
    assert_eq!(ShortString::MAX_CAPACITY, 23);

    let mut s = ShortString::new();
    assert_eq!(s.len(), 0);

    let tail = "Hello, world!";
    assert_eq!(tail.len(), 13);
    s.push_str(tail);
    assert_eq!(s.len(), 13);

    let tail = " It's hot.";
    assert_eq!(tail.len(), 10);
    s.push_str(tail);
    assert_eq!(s.len(), 23);

    assert_eq!(s.as_str(), "Hello, world! It's hot.");

    s.push_str("something else");
    assert_eq!(s.as_str(), "Hello, world! It's hot.");
}

#[test]
fn short_string_64_upgrades_correctly() {
    let mut s = ShortString::new();
    s.push_str("Hello, world!");

    for capacity in (0..32).step_by(8) {
        let mut ls = s.into_long(capacity);
        assert_eq!(ls.len(), s.len());
        assert_eq!(ls.as_str(), s.as_str());
        assert!(ls.capacity() >= ShortString::MAX_CAPACITY + capacity);
        ls.free();
    }
}

#[test]
fn long_string_extends_within_capacity() {
    let mut s = LongString::with_capacity(16);
    let capacity = s.capacity();
    assert!(capacity >= 16);

    let tail_1 = "I know that";
    assert_eq!(tail_1.len(), 11);
    s.push_str(tail_1);
    assert_eq!(s.as_str(), tail_1);

    assert_eq!(s.capacity() - tail_1.len(), s.remaining_capacity());
    let tail_2 = ".".repeat(s.remaining_capacity());
    s.push_str(&tail_2);

    let mut correct_s = StdString::from(tail_1);
    correct_s.push_str(&tail_2);

    assert_eq!(s.as_str(), &correct_s);
    assert_eq!(s.remaining_capacity(), 0);
    assert_eq!(s.capacity(), capacity);

    s.free();
}

#[test]
fn long_string_can_clone_with_additional_capacity() {
    let mut s = LongString::with_capacity(16);
    s.push_str("Hello, world!");
    let mut cloned = s.clone_with_additional_capacity(16);
    assert_eq!(s.len(), cloned.len());
    assert_eq!(s.as_str(), cloned.as_str());
    assert!(cloned.capacity() >= s.capacity() + 16);
    s.free();
    cloned.free();
}

#[test]
fn long_string_reallocs_automatically() {
    let mut s = LongString::with_capacity(16);
    let initial_capacity = s.capacity();
    let tail = ".".repeat(s.remaining_capacity());
    s.push_str(&tail);
    let len = s.len();
    assert_eq!(s.remaining_capacity(), 0);
    assert_eq!(s.len(), tail.len());

    s.push_str(".");
    assert!(s.capacity() > initial_capacity);
    assert_eq!(len + 1, s.len());

    let mut correct_s = StdString::from(&tail);
    correct_s.push_str(".");
    assert_eq!(s.as_str(), &correct_s);
    s.free();
}

#[test]
fn as_mut_str_works() {
    let mut s = String::from("Hello, world!");
    s.as_mut_str().make_ascii_uppercase();
    assert!(s.is_short());
    assert_eq!(s.as_str(), "HELLO, WORLD!");

    s.reserve(100);
    assert!(s.is_long());
    s.as_mut_str().make_ascii_lowercase();
    assert_eq!(s.as_str(), "hello, world!");
}

#[test]
fn contructable_from_raw_parts() {
    let mut s = ManuallyDrop::new(String::from("Hello, world!"));
    s.reserve(100);
    assert!(s.is_long());

    let TaggedSsoString64Mut::Long(long) = s.tagged_mut() else {
        unreachable!()
    };

    let (buf, length, capacity) = (long.buf().data, long.len(), long.capacity());
    // SAFETY: `s` is ManuallyDrop, so we own this buffer. `s` will also never call free, since we
    // override the name, so soundness of code remains the same
    let s = unsafe { String::from_raw_parts(buf.as_ptr(), length, capacity) };
    assert_eq!(&s, "Hello, world!");
}

#[test]
fn long_string_get_non_null_slice() {
    let mut long = LongString::with_capacity(32);
    long.push_str("Hello, world!");
    assert_eq!(None, long.get_non_null_slice(4, long.capacity() + 10));
    let slice = long.get_non_null_slice(0, 5).expect("valid slice");
    assert_aligned(slice.as_ptr() as *mut u8);
    assert_eq!(
        slice.as_ptr() as *const [u8],
        &long.as_bytes()[0..5] as *const [u8]
    );
    long.free();
}

#[test]
fn long_string_gets_non_null() {
    let mut long = LongString::with_capacity(32);
    long.push_str("012345");
    assert_eq!(
        long.get_non_null(4).map(|nn| nn.as_ptr() as *const u8),
        Some(&long.as_bytes()[4] as *const u8)
    );
    assert_eq!(None, long.get_non_null(33));
    long.free();
}

#[test]
fn long_string_constructs_from_str() {
    let long = LongString::from_str("💁👌🎍😍");
    assert_eq!(long.as_str(), "💁👌🎍😍");
}

#[test]
fn can_use_sso_str_for_cow() {
    let mut sso_cow = Cow::Borrowed(SsoStr::from_str("Hello, world!"));
    sso_cow.to_mut().push_str(" let's add some more stuff");
    assert_eq!(
        Cow::Owned::<SsoStr>(String::from("Hello, world! let's add some more stuff")),
        sso_cow
    );
}

#[test]
fn short_string_push_works() {
    let mut s = ShortString::new();
    s.push('a');
    assert_eq!(s.as_str(), "a");

    s.push('あ');
    assert_eq!(s.as_str(), "aあ");

    s.push_str("much too long to fit in a short string");
    assert_eq!(s.as_str(), "aあ");
}

#[test]
fn retain_works() {
    let mut s = String::from("Hello, world! I am Gregory.");
    s.retain(|ch| ch != ' ');
    assert_eq!(&s, "Hello,world!IamGregory.")
}

#[test]
fn pop_works() {
    let mut s = String::from("Hi!");
    assert_eq!(Some('!'), s.pop());
    assert_eq!(Some('i'), s.pop());
    assert_eq!(Some('H'), s.pop());
    assert_eq!(None, s.pop());
}

#[test]
fn push_works() {
    let mut s = String::from("Hello, world");
    s.push('!');
    assert_eq!(&s, "Hello, world!");

    let mut s = String::from("こんにちは世界");
    s.push('！');
    assert_eq!(&s, "こんにちは世界！")
}

#[test]
fn boost_coverage() {
    let s = String::new();
    assert!(s.is_short());
    assert_eq!(s.capacity(), ShortString::MAX_CAPACITY);
}
