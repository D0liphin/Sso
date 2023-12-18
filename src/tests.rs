type StdString = std::string::String;
type String = super::SsoString;
type ShortString = super::ShortString64;
type LongString = super::LongString;

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
