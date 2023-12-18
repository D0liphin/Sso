use std::{
    alloc::{Allocator, Global, Layout},
    cmp, fmt,
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::Deref,
    ptr::{NonNull, self},
    slice,
    str::{CharIndices, Chars},
};

#[cfg(test)]
mod tests;

fn maybe_uninit_slice<T: Sized>(sl: &[T]) -> &[MaybeUninit<T>] {
    unsafe { mem::transmute(sl) }
}

fn non_null_slice<T: Sized>(sl: &mut [MaybeUninit<T>]) -> NonNull<[T]> {
    unsafe { mem::transmute(NonNull::from(sl)) }
}

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

    /// SAFETY: `len` must be the exact length of the allocated object, using the value returned
    /// with `RawBuf::new() -> (_, len)` will guarantee safety.
    ///
    /// Returns [`InvalidArgumentError`] if `len` is impossibly big.
    pub unsafe fn dealloc(&mut self, len: usize) -> Result<(), InvalidArgumentError> {
        Global.deallocate(self.data.cast(), Self::new_layout(len).0);
        Ok(())
    }
}

#[repr(C)]
struct ShortString64 {
    len_and_flag: u8,
    buf: [u8; Self::MAX_CAPACITY],
}

impl ShortString64 {
    const MAX_CAPACITY: usize = 23;

    /// Constructs and empty ShortString64
    pub const fn new() -> Self {
        Self {
            len_and_flag: 1, // 0, 1
            buf: [0; 23],
        }
    }

    pub const fn is_short(&self) -> bool {
        (self.len_and_flag & 1) == 1
    }

    /// SAFETY: len must be less than or equal to `ShortString64::MAX_CAPACITY`
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len_and_flag = (self.len_and_flag & 1) | ((len as u8) << 1);
    }

    pub const fn len(&self) -> usize {
        ((self.len_and_flag & (u8::MAX << 1)) >> 1) as usize
    }

    pub const fn remaining_capacity(&self) -> usize {
        Self::MAX_CAPACITY - self.len()
    }

    pub fn push_str(&mut self, s: &str) {
        unsafe { self.push_bytes(s.as_bytes()) };
    }

    pub unsafe fn push_bytes(&mut self, bytes: &[u8]) {
        let count = cmp::min(bytes.len(), self.remaining_capacity());
        let mut i = self.len();
        for &b in &bytes[0..count] {
            self.buf[i] = b;
            i += 1;
        }
        // SAFETY: len is at most self.len() + self.remaining_capacity(), which is by definition
        // Self::MAX_CAPACITY
        self.set_len(self.len() + count);
    }

    /// Converts this to a [`LongString`]. Where the capacity is equal to
    /// `Self::MAX_CAPACITY + additional_capacity`.
    pub fn into_long(&self, additional_capacity: usize) -> LongString {
        let len = self.len();
        let mut long = LongString::with_capacity(Self::MAX_CAPACITY + additional_capacity);
        // SAFETY: guaranteed to have at least Self::MAX_CAPACITY elements, since we just allocated
        // at least that much
        let long_buf = unsafe { long.get_maybe_uninit_slice_unchecked_mut(0, Self::MAX_CAPACITY) };
        long_buf[0..len].copy_from_slice(&MaybeUninit::new(self.buf).transpose()[0..len]);
        // SAFETY: we have just initialised the first len bytes
        unsafe {
            long.set_len(len);
        }
        long
    }

    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self as *const u8;
        let len = self.len();
        // SAFETY: ptr add is within the object and len is at most sizeof(ShortString) - 1
        unsafe { slice::from_raw_parts(ptr.add(1), len) }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: always valid utf-8
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

#[repr(C)]
struct LongString {
    len: usize,
    buf: RawBuf<u8>,
    capacity: usize,
}

impl LongString {
    /// Construct a new `LongString` with at least `capacity` as the `capacity`. Note that this 
    /// will panic in the case of an impossible allocation (e.g. `capacity > isize::MAX`)
    pub fn with_capacity(capacity: usize) -> Self {
        let (buf, capacity) = RawBuf::new(capacity);

        Self {
            len: 0,
            buf,
            capacity,
        }
    }

    /// interpret this as a `&str`
    pub fn as_str(&self) -> &str {
        // SAFETY: `LongString` always contains valid utf-8, buf[0..len] is always initialised 
        unsafe {
            std::str::from_utf8_unchecked(self.as_bytes())
        }
    }

    /// alias for `self.as_str().as_bytes()`
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: 
        // - valid for reads of u8, since we are within 0..len, which is by definition, 
        //   initialised and allocated
        // - we cannot mutate the slice, since the returned slice lives as long as the borrow
        //   to self
        // - allocations are no larger than isize::MAX, so len can never be greater than that
        unsafe {
            slice::from_raw_parts(self.buf.data.as_ptr(), self.len())
        }
    }

    /// Replaces the contents of the buffer with `s`, so that `&self == s[0..self.len()]`.
    ///
    /// Note that `s` is truncated to fit inside a buffer of size `self.capacity()`, you should
    /// check first if `s` can fit, if you would like to guarantee that `&self == s`
    pub fn copy_from(&mut self, s: &str) {
        // This doesn't violate the function's description, as this simply means that s cannot
        // fit in the buffer at all, hence the contract &self == s[0..self.len()] is fulfilled in
        // the form "" == s[0..0]
        let dst = self.get_sized_buf_mut();
        let count = cmp::min(s.len(), self.capacity());
        unsafe {
            // SAFETY:
            // - src is valid for reads, since count <= s.len()
            // - dst os valid for writes of count bytes, since count <= s.capacity()
            // - both dst and src are cast from aligned pointers
            // - the regions may not overlap, as `&mut` takes unique ownership of the bufer, thus
            //   `&str` must point somewhere else
            ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), dst.as_mut_ptr(), count);
            // SAFETY: we just initialised count bytes starting at 0 with a valid str
            self.set_len(count);
        }
    }

    /// Returns a sized buffer representing the whole buffer of the string, can be safely written to
    /// so long as utf-8 constraints are not invalidated, and the buffer is not resized
    pub fn get_sized_buf_mut(&mut self) -> NonNull<[u8]> {
        unsafe {
            // SAFETY:
            // - region specified is allocated and within the same allocation, since it is always
            //   within RawBuf.data
            // - region specified is valid for writes, because it is SharedReadWrite
            let buf = ptr::slice_from_raw_parts_mut(self.buf.data.as_ptr(), self.capacity());
            // SAFETY:
            // - `raw` is constructed froma a NonNull and is thus valid to cast to a NonNull
            NonNull::new_unchecked(buf)
        }
    }

    pub fn get_non_null_slice_mut(&mut self, index: usize, len: usize) -> Option<NonNull<[u8]>> {
        if index + len > self.capacity() {
            return None;
        }
        let Some(data) = self.get_non_null_mut(index) else { return None; };

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

    /// get unchecked [`NonNull<u8>`] to an index in the buffer, use `get_non_null_mut` for a safe 
    /// version of this function
    /// 
    /// # Safety
    /// - You must uphold `index <= self.capacity()`
    pub unsafe fn get_non_null_unchecked_mut(&mut self, index: usize) -> NonNull<u8> {
        // SAFETY:
        // - the maxmimum index is capacity, which is within the specified boundary of the allocated
        //   object (RawBuf.data), or one byte past the end
        // - we cannot allocate a buffer of more than isize::MAX, thus capacity must be less than 
        //   `isize::MAX`
        // - allocations are fully within the address space, so we cannot wrap around
        let mut ptr = self.buf.data.as_ptr().add(index);
        // SAFETY:
        // - valid ptr.add() on a valid NonNull is guaranteed to produce a valid NonNull
        NonNull::new_unchecked(ptr)
    }

    /// returns a pointer to the element of the buffer that is at an offset of `index` from the
    /// start, or `None` if the pointer is out of bounds
    pub fn get_non_null_mut(&mut self, index: usize) -> Option<NonNull<u8>> {
        if index > self.capacity() {
            return None;
        }

        // SAFETY: exact required bounds check performed, no mutations following bounds check
        unsafe {
            Some(self.get_non_null_unchecked_mut(index))
        }
    }

    /// returns a pointer to the next element of the buffer that we want to allocate to, note that 
    /// the pointer might not be writeable, as it could be outside of the buffer. In order to write
    /// to the pointer, ensure that `len < capacity`
    pub fn next_ptr(&mut self) -> NonNull<u8> {
        // SAFETY: len, by definition, always satisfies `len <= capacity`
        unsafe {
            self.get_non_null_unchecked_mut(self.len())
        }
    }

    /// returns the length of this string in bytes, length upholds the following invariants, that 
    /// you needn't check
    /// 
    /// - `self.len() < self.capacity()`
    /// - `self.len() < isize::MAX` (derived invariant from `self.capacity() < isize::MAX`)
    pub const fn len(&self) -> usize {
        self.len >> 1
    }

    /// Returns the capacity of this string, that is, how many bytes it can fit before a realloc.
    /// Note that this does not mean *extra bytes*, but total bytes. Use `remaining_capacity` for 
    /// that.
    /// 
    /// `self.capacity()` upholds the following invariants:
    /// 
    /// - `self.capacity() < isize::MAX`
    pub const fn capacity(&self) -> usize {
        self.capacity
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
                self.remaining_capacity()
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
        self.len = len << 1;
    }

    /// free the buffer of this String
    /// 
    /// # Safety 
    /// you must never use this after calling this function
    pub unsafe fn free(&mut self) {
        // SAFETY: as per the guidelines of the function, `self.capacity()` is just what is
        // returned on the buffer's creation
        unsafe {
            self.buf
                .dealloc(self.capacity())
                .expect("should be the exact capacity");
        }
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

impl Drop for LongString {
    fn drop(&mut self) {
        // SAFETY: never used after this
        unsafe { self.free() }
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

`#[cfg(target_endian = "little")]
#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub union SsoString {
    short: ManuallyDrop<ShortString64>,
    long: ManuallyDrop<LongString>,
}`

impl Drop for SsoString {
    fn drop(&mut self) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Long(long) => {
                drop(long);
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

    fn tagged_mut(&mut self) -> TaggedSsoString64Mut {
        if self.is_short() {
            TaggedSsoString64Mut::Short(unsafe { &mut self.short })
        } else {
            TaggedSsoString64Mut::Long(unsafe { &mut self.long })
        }
    }

    fn tagged(&self) -> TaggedSsoString64 {
        if self.is_short() {
            TaggedSsoString64::Short(unsafe { &self.short })
        } else {
            TaggedSsoString64::Long(unsafe { &self.long })
        }
    }

    /// SAFETY: `s` must be valid utf-8
    unsafe fn push_bytes(&mut self, bytes: &[u8]) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Short(short) => {
                if short.remaining_capacity() >= bytes.len() {
                    short.push_bytes(bytes);
                } else {
                    let mut long = ManuallyDrop::new(short.into_long(bytes.len()));
                    long.push_bytes(bytes);
                    *self = SsoString { long };
                }
            }
            TaggedSsoString64Mut::Long(long) => {
                long.push_bytes(bytes);
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

    pub fn push_str(&mut self, s: &str) {
        unsafe { self.push_bytes(s.as_bytes()) }
    }

    pub fn len(&self) -> usize {
        match self.tagged() {
            TaggedSsoString64::Short(short) => short.len(),
            TaggedSsoString64::Long(long) => long.len(),
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
