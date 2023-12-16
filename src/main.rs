#![feature(
    vec_push_within_capacity,
    allocator_api,
    alloc_layout_extra,
    slice_ptr_get,
    maybe_uninit_uninit_array_transpose
)]

use std::{
    alloc::{Allocator, Global, Layout},
    cmp, fmt,
    mem::{self, ManuallyDrop, MaybeUninit},
    ptr::NonNull,
    slice, str::{Chars, CharIndices},
};

fn maybe_uninit_slice<T: Sized>(sl: &[T]) -> &[MaybeUninit<T>] {
    unsafe { mem::transmute(sl) }
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
    pub fn with_capacity(capacity: usize) -> Self {
        let (buf, capacity) = RawBuf::new(capacity);

        Self {
            len: 0,
            buf,
            capacity,
        }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: `LongString` always contains valid utf-8, buf[0..len] is always valid
        unsafe { mem::transmute(self.as_maybe_uninit_slice()) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    /// SAFETY: you must guarantee to keep the bytes as valid utf-8
    pub unsafe fn as_maybe_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.get_maybe_uninit_slice_unchecked_mut(0, self.len())
    }

    pub fn as_maybe_uninit_slice(&self) -> &[MaybeUninit<u8>] {
        // SAFETY: by definition
        unsafe { self.get_maybe_uninit_slice_unchecked(0, self.len()) }
    }

    /// SAFETY: the requested slice must be within the allocated buffer, that is to say that
    ///
    /// - `start < self.capacity()`
    /// - `start + len < self.capacity()`
    unsafe fn get_maybe_uninit_slice_unchecked_mut(
        &mut self,
        start: usize,
        len: usize,
    ) -> &mut [MaybeUninit<u8>] {
        slice::from_raw_parts_mut(self.get_maybe_uninit_unchecked_mut(start) as *mut _, len)
    }

    /// SAFETY: the requested slice must be within the allocated buffer, that is to say that
    ///
    /// - `start < self.capacity()`
    /// - `start + len < self.capacity()`
    unsafe fn get_maybe_uninit_slice_unchecked(
        &self,
        start: usize,
        len: usize,
    ) -> &[MaybeUninit<u8>] {
        slice::from_raw_parts(self.get_maybe_uninit_unchecked(start) as *const _, len)
    }

    /// SAFETY: `index` must be less than `self.capacity`
    unsafe fn get_maybe_uninit_unchecked_mut(&mut self, index: usize) -> &mut MaybeUninit<u8> {
        let ptr = self.buf.data.as_ptr().add(index) as *mut MaybeUninit<u8>;
        // SAFETY: guaranteed to be aligned, allocated, exclusive and live as long as 'self.
        &mut *ptr
    }

    /// SAFETY: `index` must be less than `self.capacity`
    unsafe fn get_maybe_uninit_unchecked(&self, index: usize) -> &MaybeUninit<u8> {
        let ptr = self.buf.data.as_ptr().add(index) as *const MaybeUninit<u8>;
        // SAFETY: guaranteed to be aligned, allocated, exclusive and live as long as 'self.
        &*ptr
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// SAFETY: must not modify the resulting byte such that the string becomes invalid utf-8
    unsafe fn get_within_capacity_mut(&mut self, index: usize) -> Option<&mut MaybeUninit<u8>> {
        if index >= self.capacity {
            return None;
        }

        // SAFETY: guaranteed to be within the object due to bounds check, allocating objects
        // beyond the address space will fail anyway
        Some(unsafe { self.get_maybe_uninit_unchecked_mut(index) })
    }

    /// `len` is truncated to a 63-bit number  
    ///
    /// SAFETY: the function itself is never unsafe, but could make some safe functions that depend
    /// on the length being valid unsafe. Ensure that everything within 0..len is allocated and
    /// initialised.
    unsafe fn set_len(&mut self, len: usize) {
        self.len = len << 1;
    }

    pub const fn len(&self) -> usize {
        self.len >> 1
    }

    pub const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    pub fn clone_with_additional_capacity(&self, additional_capacity: usize) -> Self {
        let mut new = Self::with_capacity(self.capacity() + additional_capacity);
        // SAFETY: copying valid utf-u8 from another LongString
        let dst = unsafe { new.as_maybe_uninit_slice_mut() };
        let src = self.as_maybe_uninit_slice();
        dst[0..src.len()].copy_from_slice(src);
        // SAFETY: we just initialised self.len() bytes of data
        unsafe { new.set_len(self.len()) }
        new
    }

    /// realloc to fit at least `remaining_capacity` more bytes
    pub fn realloc(&mut self, remaining_capacity: usize) {
        *self = self.clone_with_additional_capacity(cmp::max(
            remaining_capacity - self.remaining_capacity(),
            self.capacity() * 2,
        ));
    }

    pub unsafe fn push_bytes(&mut self, bytes: &[u8]) {
        if self.remaining_capacity() < bytes.len() {
            self.realloc(bytes.len());
        }
        unsafe {
            // SAFETY: we have allocated at least bytes.len() extra space
            let dst = self.get_maybe_uninit_slice_unchecked_mut(self.len(), bytes.len());
            // SAFETY: src is valid utf-8
            dst.copy_from_slice(maybe_uninit_slice(bytes));
            // SAFETY: just initialised bytes.len() extra space
            self.set_len(self.len() + bytes.len());
        }
    }

    /// SAFETY: you must never use this after calling this function
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

#[cfg(target_endian = "little")]
#[cfg(target_pointer_width = "64")]
#[repr(C)]
union SsoString {
    short: ManuallyDrop<ShortString64>,
    long: ManuallyDrop<LongString>,
}

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

impl SsoString {
    pub fn new() -> Self {
        Self {
            short: ManuallyDrop::new(ShortString64::new()),
        }
    }

    pub fn is_short(&self) -> bool {
        // SAFETY: transmuting anything to a byte array is always valid, which is essentially what
        // we're doing when we do this.
        unsafe { self.short.is_short() }
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

type Str = SsoString;

fn main() {
    let mut string = Str::new();

    string.push_str("Hello, world!");
    println!("{:?}", string);

    string.push_str(" My name i");
    println!("{:?}", string);

    string.push_str("s Greg!");
    println!("{:?}", string);
}
