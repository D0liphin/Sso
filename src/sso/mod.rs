use std::{
    alloc::{Allocator, Global, Layout},
    cmp, fmt,
    mem::ManuallyDrop,
    ops::{self, Deref},
    ptr::{self, NonNull},
    slice,
    str::{CharIndices, Chars},
};

use crate::unsafe_field::UnsafeWrite;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
#[repr(C)]
struct RawBuf<T> {
    data: NonNull<T>,
}

#[derive(Debug)]
struct InvalidArgumentError;

impl<T> RawBuf<T> {
    const fn dangling() -> Self {
        Self {
            data: NonNull::dangling(),
        }
    }

    fn new_layout(capacity: usize) -> (Layout, usize) {
        Layout::new::<T>()
            .repeat(capacity)
            .expect("capacity should be valid")
    }

    pub fn new(capacity: usize) -> (Self, usize) {
        if capacity == 0 {
            return (Self::dangling(), 0);
        }

        let (layout, offset) = Self::new_layout(capacity);
        let data = Global
            .allocate(layout)
            .expect("should be a valid allocation");
        let capacity = data.len() / offset; // offset is the size of each allocation with padding
        let data = data.as_non_null_ptr();

        (Self { data: data.cast() }, capacity)
    }

    /// Deallocates the buffer. Returns [`InvalidArgumentError`] if `len` is impossibly big.
    ///
    /// # Safety
    /// - `len` must be the exact length of the allocated object, using the value returned
    /// with `RawBuf::new() -> (_, len)` will guarantee safety.
    /// - this must be the first time that you call this function
    pub unsafe fn dealloc(self, len: usize) -> Result<Self, InvalidArgumentError> {
        Global.deallocate(self.data.cast(), Self::new_layout(len).0);
        Ok(self)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ShortString64 {
    /// # Safety
    /// - `1` is always a valid value
    /// - the last bit must always be `1`
    /// when shifted by >> 1:
    /// - `len` must be less than or equal to `ShortString64::MAX_CAPACITY`
    len_and_flag: UnsafeWrite<u8, 0>,
    buf: [u8; Self::MAX_CAPACITY],
}

impl ShortString64 {
    const MAX_CAPACITY: usize = 23;

    /// Constructs and empty ShortString64
    pub const fn new() -> Self {
        Self {
            // SAFETY: 1 is always a valid value
            len_and_flag: unsafe { UnsafeWrite::new(1) }, // 0, 1
            buf: [0; 23],
        }
    }

    /// in a union with a long string, returns `true` if this has been upgraded
    pub const fn is_short(&self) -> bool {
        (*self.len_and_flag.get() & 1) == 1
    }

    /// Although not unsafe, sa the string is zeroed, you shold uphold that `len` is all
    /// user-initialised. This depends on the function that you are implementing with this.
    ///
    /// SAFETY:
    /// - `len` must be less than or equal to `ShortString64::MAX_CAPACITY`
    pub unsafe fn set_len(&mut self, len: usize) {
        let mask = *self.len_and_flag.get() & 1;
        // SAFETY:
        // - len is masked `mask` which sets the last bit to 1 no matter what and does not affect
        //   the first 7 bits at all
        // - len >> 1 is len, the safety contract is passed to the caller
        self.len_and_flag.set(mask | ((len as u8) << 1));
    }

    /// Returns the length of this short string, `len` upholds fewer invariants on a short string,
    /// than on a long, these are
    ///
    /// - `self.len() <= self.capacity()` Note that `self.capacity()` is a constant
    pub const fn len(&self) -> usize {
        ((*self.len_and_flag.get() & (u8::MAX << 1)) >> 1) as usize
    }

    /// Returns the capacity of this short string, this is a constant, which is equal to
    /// [`Self::MAX_CAPACITY`]
    pub const fn capacity(&self) -> usize {
        Self::MAX_CAPACITY
    }

    pub const fn remaining_capacity(&self) -> usize {
        Self::MAX_CAPACITY - self.len()
    }

    pub fn push_str(&mut self, s: &str) {
        let count = cmp::min(s.len(), self.remaining_capacity());
        let mut i = self.len();
        for &b in &s.as_bytes()[0..count] {
            self.buf[i] = b;
            i += 1;
        }
        // SAFETY: len is at most self.len() + self.remaining_capacity(), which is by definition
        // Self::MAX_CAPACITY
        unsafe {
            self.set_len(self.len() + count);
        }
    }

    /// Converts this to a [`LongString`]. Where the capacity is equal to
    /// `Self::MAX_CAPACITY + additional_capacity`.
    pub fn into_long(&self, additional_capacity: usize) -> LongString {
        let mut long = LongString::with_capacity(Self::MAX_CAPACITY + additional_capacity);
        long.copy_from(self.as_str());
        long
    }

    /// Returns a slice of bytes that is always valid utf-8
    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self as *const u8;
        // SAFETY:
        // - ptr add is within the object and len is at most sizeof(ShortString) - 1
        // - note that we construct this from a `&self`, to comply with Stacked Borrows
        unsafe { slice::from_raw_parts(ptr.add(1), self.len()) }
    }

    /// interpret this string as a `&str`
    pub fn as_str(&self) -> &str {
        // SAFETY: always valid utf-8, by definition
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl fmt::Display for ShortString64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for ShortString64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

// SAFETY: all structs contain different integers
#[repr(C)]
struct LongString {
    /// # Safety
    /// - `0` is always a valid value
    /// - the last bit is always 0
    /// when shifted by >> 1:
    /// - `len <= capacity`
    /// - `buf[0..len]` is always a valid SharedReadWrite slice of valid u8, if the string is not
    ///    borrowed, otherwise the permissions become that of the borrow
    len: UnsafeWrite<usize, 0>,
    /// SAFETY:
    /// buf and capacity are linked, so we can only modify either if we update the entire struct
    /// simultaneously. As a result, we cannot implement Drop. The size of the allocated object
    /// starting at buf.data is always exactly capacity bytes long.
    buf: UnsafeWrite<RawBuf<u8>, 1>,
    /// SAFETY:
    /// buf and capacity are linked, so we can only modify either if we update the entire struct
    /// simultaneously. As a result, we cannot implement Drop. The size of the allocated object
    /// starting at buf.data is always exactly capacity bytes long.
    capacity: UnsafeWrite<usize, 2>,
}

impl LongString {
    /// Construct a new `LongString` with at least `capacity` as the `capacity`. Note that this
    /// will panic in the case of an impossible allocation (e.g. `capacity > isize::MAX`)
    pub fn with_capacity(capacity: usize) -> Self {
        let (buf, capacity) = RawBuf::new(capacity);

        unsafe {
            Self {
                // SAFETY: a value of `0` is always valid
                len: UnsafeWrite::new(0),
                // SAFETY: by definition of RawBuf::new, capacity and buf match, so both these
                // constructions are safe
                capacity: UnsafeWrite::new(capacity),
                buf: UnsafeWrite::new(buf),
            }
        }
    }

    /// interpret this as a `&str`
    pub fn as_str(&self) -> &str {
        // SAFETY: `LongString` always contains valid utf-8, buf[0..len] is always initialised
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// alias for `self.as_str().as_bytes()`
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY:
        // - valid for reads of u8, since we are within 0..len, which is by definition,
        //   initialised and allocated
        // - we cannot mutate the slice, since the returned slice lives as long as the borrow
        //   to self
        // - allocations are no larger than isize::MAX, so len can never be greater than that
        unsafe { slice::from_raw_parts(self.buf().data.as_ptr(), self.len()) }
    }

    /// Replaces the contents of the buffer with `s`, so that `&self == s[0..self.len()]`.
    ///
    /// Note that `s` is truncated to fit inside a buffer of size `self.capacity()`, you should
    /// check first if `s` can fit, if you would like to guarantee that `&self == s`
    pub fn copy_from(&mut self, s: &str) {
        // This doesn't violate the function's description, as this simply means that s cannot
        // fit in the buffer at all, hence the contract &self == s[0..self.len()] is fulfilled in
        // the form "" == s[0..0]
        let dst = self.get_sized_buf();
        let count = cmp::min(s.len(), self.capacity());
        unsafe {
            // SAFETY:
            // - src is valid for reads, since count <= s.len()
            // - dst os valid for writes of count bytes, since count <= s.capacity()
            // - both dst and src are cast from aligned pointers
            // - the regions may not overlap, as `&mut` takes unique ownership of the bufer, thus
            //   `&str` must point somewhere else
            ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), dst.as_mut_ptr(), count);
            // SAFETY:
            // - we just initialised count bytes starting at 0 with a valid str
            // - count is at most capacity, so len <= capacity
            self.set_len(count);
        }
    }

    /// Returns a sized buffer representing the whole buffer of the string, can be safely written to
    /// so long as utf-8 constraints are not invalidated, and the buffer is not resized
    pub fn get_sized_buf(&self) -> NonNull<[u8]> {
        unsafe {
            // SAFETY:
            // - region specified is allocated and within the same allocation, since it is always
            //   within RawBuf.data
            // - region specified is valid for writes, because it is SharedReadWrite
            let buf = ptr::slice_from_raw_parts_mut(self.buf().data.as_ptr(), self.capacity());
            // SAFETY:
            // - `raw` is constructed froma a NonNull and is thus valid to cast to a NonNull
            NonNull::new_unchecked(buf)
        }
    }

    #[allow(dead_code)]
    pub fn get_non_null_slice(&self, index: usize, len: usize) -> Option<NonNull<[u8]>> {
        if index + len > self.capacity() {
            return None;
        }
        let Some(data) = self.get_non_null(index) else { return None; };

        unsafe {
            // SAFETY:
            // - region specified is allocated and within the same allocation, since it is always
            //   within RawBuf.data
            // - region specified is valid for writes, because it is SharedReadWrite
            let raw = ptr::slice_from_raw_parts_mut(data.as_ptr(), len);
            // SAFETY:
            // - `raw` is constructed froma a NonNull and is thus valid to cast to a NonNull
            Some(NonNull::new_unchecked(raw))
        }
    }

    /// get unchecked [`NonNull<u8>`] to an index in the buffer, use `get_non_null` for a safe
    /// version of this function
    ///
    /// # Safety
    /// - You must uphold `index <= self.capacity()`
    pub unsafe fn get_non_null_unchecked(&self, index: usize) -> NonNull<u8> {
        // SAFETY:
        // - the maxmimum index is capacity, which is within the specified boundary of the allocated
        //   object (RawBuf.data), or one byte past the end
        // - we cannot allocate a buffer of more than isize::MAX, thus capacity must be less than
        //   `isize::MAX`
        // - allocations are fully within the address space, so we cannot wrap around
        let ptr = self.buf().data.as_ptr().add(index);
        // SAFETY:
        // - valid ptr.add() on a valid NonNull is guaranteed to produce a valid NonNull
        NonNull::new_unchecked(ptr)
    }

    /// returns a pointer to the element of the buffer that is at an offset of `index` from the
    /// start, or `None` if the pointer is out of bounds
    pub fn get_non_null(&self, index: usize) -> Option<NonNull<u8>> {
        if index > self.capacity() {
            return None;
        }

        // SAFETY: exact required bounds check performed, no mutations following bounds check
        unsafe { Some(self.get_non_null_unchecked(index)) }
    }

    /// returns a pointer to the next element of the buffer that we want to allocate to, note that
    /// the pointer might not be writeable, as it could be outside of the buffer. In order to write
    /// to the pointer, ensure that `len < capacity`
    pub fn next_ptr(&mut self) -> NonNull<u8> {
        // SAFETY: len, by definition, always satisfies `len <= capacity`
        unsafe { self.get_non_null_unchecked(self.len()) }
    }

    /// returns the length of this string in bytes, length upholds the following invariants, that
    /// you needn't check
    ///
    /// - `self.len() < self.capacity()`
    /// - `self.len() < isize::MAX` (derived invariant from `self.capacity() < isize::MAX`)
    pub const fn len(&self) -> usize {
        *self.len.get() >> 1
    }

    /// Returns the capacity of this string, that is, how many bytes it can fit before a realloc.
    /// Note that this does not mean *extra bytes*, but total bytes. Use `remaining_capacity` for
    /// that.
    ///
    /// `self.capacity()` upholds the following invariants:
    ///
    /// - `self.capacity() < isize::MAX`
    /// - `self.capacity()` is the exact size of the allocated buffer
    pub const fn capacity(&self) -> usize {
        *self.capacity.get()
    }

    /// Gets the underyling buffer being used for this string
    pub const fn buf(&self) -> &RawBuf<u8> {
        self.buf.get()
    }

    /// returns the remaining capacity of this string (how many bytes we can allcoate before a
    /// realloc must occur)
    pub const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    /// clones this string, with at least `additional_capacity` extra space
    pub fn clone_with_additional_capacity(&self, additional_capacity: usize) -> Self {
        let mut new = Self::with_capacity(self.capacity() + additional_capacity);
        new.copy_from(self.as_str());
        new
    }

    /// realloc to fit at least `remaining_capacity` more bytes
    pub fn realloc(&mut self, remaining_capacity: usize) {
        *self = self.clone_with_additional_capacity(cmp::max(
            remaining_capacity - self.remaining_capacity(),
            self.capacity() * 2,
        ));
    }

    /// Push a `str` to this string, allocating if needed. Note that the current realloc schema
    /// might only allocate exactly enough extra space for `s`
    pub fn push_str(&mut self, s: &str) {
        let str_len = s.as_bytes().len();
        if self.remaining_capacity() < s.len() {
            self.realloc(str_len);
        }

        unsafe {
            // SAFETY:
            // - src (s) is valid for reads of str_len by slice definition
            // - dst is valid for writes of count str_len, since remaining_capacity >= str_len
            // - both dst and src are cast from aligned pointers
            // - the regions may not overlap, as `&mut` takes unique ownership of the buffer, thus
            //   `&str` must point somewhere else
            ptr::copy_nonoverlapping(
                s.as_bytes().as_ptr(),
                self.next_ptr().as_ptr(),
                self.remaining_capacity(),
            );
            // SAFETY: just copied a valid str of str_len into the section starting at len
            self.set_len(self.len() + str_len);
        }
    }

    /// `len` is truncated to a 63-bit number.
    ///
    /// # Safety
    /// - everything from `buf[0..len]` must be initialised.
    /// - you must uphold `len <= capacity`
    unsafe fn set_len(&mut self, len: usize) {
        // SAFETY: safety contract passed to caller
        self.len.set(len << 1);
    }

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
}

impl fmt::Display for LongString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for LongString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl Clone for LongString {
    fn clone(&self) -> Self {
        self.clone_with_additional_capacity(0)
    }
}

enum TaggedSsoString64Mut<'a> {
    Short(&'a mut ShortString64),
    Long(&'a mut LongString),
}

enum TaggedSsoString64<'a> {
    Short(&'a ShortString64),
    Long(&'a LongString),
}

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
#[repr(C)]
pub union SsoString {
    short: ManuallyDrop<ShortString64>,
    long: ManuallyDrop<LongString>,
}

impl Drop for SsoString {
    fn drop(&mut self) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Long(long) => {
                long.free();
            }
            _ => {}
        }
    }
}

impl<'a> From<&'a str> for SsoString {
    fn from(value: &'a str) -> Self {
        let mut s = Self::new();
        s.push_str(value);
        s
    }
}

impl SsoString {
    pub fn new() -> Self {
        Self {
            short: ManuallyDrop::new(ShortString64::new()),
        }
    }

    /// Returns `true` if this string is a short string (no heap allocations), and `false` otherwise
    pub fn is_short(&self) -> bool {
        // SAFETY: transmuting anything to a byte array is always valid, which is essentially what
        // we're doing when we do this.
        unsafe { self.short.is_short() }
    }

    /// Returns `!self.is_short()`
    pub fn is_long(&self) -> bool {
        !self.is_short()
    }

    /// Returns the underlying union as an enum, allowing you to access the underlying short or
    /// long variant for the string
    fn tagged(&self) -> TaggedSsoString64 {
        if self.is_short() {
            TaggedSsoString64::Short(unsafe { &self.short })
        } else {
            TaggedSsoString64::Long(unsafe { &self.long })
        }
    }

    /// Same as [`SsoString::tagged`], but returns allows mutation of the underlying values instead
    fn tagged_mut(&mut self) -> TaggedSsoString64Mut {
        if self.is_short() {
            TaggedSsoString64Mut::Short(unsafe { &mut self.short })
        } else {
            TaggedSsoString64Mut::Long(unsafe { &mut self.long })
        }
    }

    /// Push a str `s` onto the end of this string
    pub fn push_str(&mut self, s: &str) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Short(short) => {
                if short.remaining_capacity() >= s.len() {
                    short.push_str(s);
                } else {
                    let mut long = ManuallyDrop::new(short.into_long(s.len()));
                    long.push_str(s);
                    *self = SsoString { long };
                }
            }
            TaggedSsoString64Mut::Long(long) => {
                long.push_str(s);
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self.tagged() {
            TaggedSsoString64::Short(short) => short.as_str(),
            TaggedSsoString64::Long(long) => long.as_str(),
        }
    }

    pub fn chars(&self) -> Chars {
        self.as_str().chars()
    }

    pub fn chars_indices(&self) -> CharIndices {
        self.as_str().char_indices()
    }

    pub fn len(&self) -> usize {
        match self.tagged() {
            TaggedSsoString64::Short(short) => short.len(),
            TaggedSsoString64::Long(long) => long.len(),
        }
    }

    pub fn capacity(&self) -> usize {
        match self.tagged() {
            TaggedSsoString64::Short(short) => short.capacity(),
            TaggedSsoString64::Long(long) => long.capacity(),
        }
    }
}

impl PartialEq<SsoString> for SsoString {
    fn eq(&self, other: &SsoString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for SsoString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<SsoString> for str {
    fn eq(&self, other: &SsoString) -> bool {
        self == other.as_str()
    }
}

impl Deref for SsoString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for SsoString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tagged() {
            TaggedSsoString64::Short(short) => write!(f, "{}", short),
            TaggedSsoString64::Long(long) => write!(f, "{}", long),
        }
    }
}

impl fmt::Debug for SsoString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tagged() {
            TaggedSsoString64::Short(short) => write!(f, "Short({:?})", short),
            TaggedSsoString64::Long(long) => write!(f, "Long({:?})", long),
        }
    }
}

impl ops::AddAssign<&str> for SsoString {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl ops::Add<&str> for SsoString {
    type Output = Self;

    fn add(mut self, rhs: &str) -> Self::Output {
        self += rhs;
        self
    }
}

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
pub type String = SsoString;

#[cfg(all(not(target_endian = "little"), not(target_pointer_width = "64")))]
pub type String = std::string::String;
