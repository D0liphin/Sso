#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "nightly", feature(allocator_api))]

#[cfg(feature = "nightly")]
use std::alloc::{Allocator, Global};

use std::{
    alloc::Layout,
    borrow::{Borrow, Cow},
    cmp,
    collections::TryReserveError,
    fmt,
    mem::{self, ManuallyDrop},
    ops::RangeBounds,
    ops::{self, Deref},
    ptr::{self, NonNull},
    slice,
    string::{Drain, FromUtf16Error, FromUtf8Error},
};

mod impl_macros;
mod unified_alloc;
pub mod unsafe_field;

use unsafe_field::{UnsafeAssign, UnsafeField};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RawBuf<T> {
    data: NonNull<T>,
}

#[derive(Debug)]
pub struct InvalidArgumentError;

impl<T> RawBuf<T> {
    pub const fn dangling() -> Self {
        Self {
            data: NonNull::dangling(),
        }
    }

    pub fn new(capacity: usize) -> (Self, usize) {
        if capacity == 0 {
            return (Self::dangling(), 0);
        }

        let data = unified_alloc::alloc_slice::<T>(capacity);

        (Self { data: data.cast() }, data.len())
    }

    /// Deallocates the buffer. Returns [`InvalidArgumentError`] if `len` is impossibly big.
    ///
    /// # Safety
    /// - `len` must be the exact length of the allocated object, using the value returned
    ///   with `RawBuf::new() -> (_, len)` will guarantee safety.
    /// - this must be the first time that you call this function (aka self.data cannot be dangling)
    pub unsafe fn dealloc(mut self, len: usize) -> Result<Self, InvalidArgumentError> {
        // SAFETY: cast temporarily for method, pointer is non-null still
        let nonnull_slice = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(self.data.as_ptr(), len))
        };
        // SAFETY:
        // - nonnull_slice points to an allocation created by RawBuf<T>
        // - the allocation should not be deallocated (caller contract)
        // - the len of the allocation should be the same as what was returned by new (caller
        //   contract)
        unsafe {
            unified_alloc::dealloc_slice(nonnull_slice);
        }
        // we need to self.data explicitly dangle, so that general slice functions are
        // perceived as safe by Miri. If we allocate, and deallocate, Miri has a tag for the
        // region and will think slice_from_raw_parts is valid.
        self.data = NonNull::dangling();
        Ok(self)
    }

    /// Returns the head of this buffer as a raw pointer, this pointer is guaranteed to be non-null
    /// aligned, and point to a valid allocation of `[T]`. The number of elements is the same as
    /// the second element that is returned by `RawBuf::new`
    ///
    /// [`RawBuf::new`]: sso::RawBuf::new
    pub const fn as_ptr(&self) -> *mut T {
        self.data.as_ptr()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
#[repr(align(8))]
pub struct ShortString64 {
    /// # Safety
    /// - `1` is always a valid value
    /// - the last bit must always be `1`
    /// when shifted by >> 1:
    /// - `len` must be less than or equal to `ShortString64::MAX_CAPACITY`
    len_and_flag: UnsafeField<u8, 0>,
    /// # Safety
    /// - Must always be valid utf8
    buf: UnsafeField<[u8; Self::MAX_CAPACITY], 1>,
}

impl ShortString64 {
    pub const MAX_CAPACITY: usize = 23;

    /// Constructs and empty ShortString64
    pub fn new() -> Self {
        Self {
            // SAFETY: 1 is always a valid value
            len_and_flag: unsafe { UnsafeField::new(1) }, // 0, 1
            buf: unsafe { UnsafeField::new([0; 23]) },
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

    /// Returns the next pointer where we should allocate our string. This validates Stacked 
    /// Borrows, by using the write access of `self`.
    /// 
    /// # Safety
    /// - the returned pointer is only writable if self.len() < self.capacity()
    /// - you must only write to this pointer if you know it is valid utf8
    pub fn next_ptr(&mut self) -> NonNull<u8> {
        // SAFETY: 
        // - no issues with overflow or invalid value as self.len() < Self::MAX_CAPACITY, which is
        //   23.
        // - ... which is also the size of the buffer, so we're either one past buf, or within 
        //   the buffer
        unsafe {
            let raw = self.buf.get_mut().cast::<u8>().as_ptr();
            // SAFETY: raw is non-null because it is 'within' a valid allocation 
            NonNull::new_unchecked(raw.add(self.len()))
        }
    }

    /// # Safety
    /// - `s.len()` must be equal to or less than `self.remaining_capacity()`
    pub unsafe fn push_str_unchecked(&mut self, s: &str) {
        // SAFETY:
        // - src is valid for reads of count s.len(), as it is s
        // - dst is valid for writes of count s.len() as s.len() self.remaining_capacity(), and
        //   buf[self.len()] will point to a buffer of size remaining_capacity()
        // - both are cast from aligned pointers
        // - both are non-overlapping, we just created new_buf on the stack
        ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), self.next_ptr().as_ptr(), s.len());
        // SAFETY:
        // - len is at most self.len() + self.remaining_capacity(), which is by definition
        //   Self::MAX_CAPACITY
        // - self.buf[len..len + s.len()] has just been initialised as valid utf8 from a str
        self.set_len(self.len() + s.len());
    }

    pub fn push_str(&mut self, s: &str) {
        let s_len = cmp::min(s.len(), self.remaining_capacity());
        if s_len == 0 {
            return;
        }
        // SAFETY: we truncate s to at most self.remaining_capacity(), therefore s_truncated is 
        // <= self.remaining_capacity();
        let s_truncated = &s[0..s_len];
        unsafe {
            self.push_str_unchecked(&s_truncated);
        }
    }

    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let utf8 = ch.encode_utf8(&mut buf);
        self.push_str(utf8);
    }

    /// Converts this to a [`LongString`]. Where the capacity is equal to or greater than
    /// `Self::MAX_CAPACITY + additional_capacity`.
    pub fn into_long(&self, additional_capacity: usize) -> LongString {
        let mut long = LongString::with_capacity(Self::MAX_CAPACITY + additional_capacity);
        // SAFETY: long has at least Self::MAX_CAPACITY space, so it can fit any string this
        // short string contains
        unsafe {
            long.push_str_unchecked(self.as_str());
        }
        long
    }

    /// Returns a slice of bytes that is always valid utf-8
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: always safe to convert to &[u8]
        unsafe { &*self.get_sized_buf().as_ptr() }
    }

    /// interpret this string as a `&str`
    pub fn as_str(&self) -> &str {
        // SAFETY: always valid utf-8, by definition
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Returns `buf[0..len]` as a `NonNull<[u8]>` with len `len`, you may cast the resulting slice
    /// to a `&'self str`  at any time, assuming all unsafe functions are called with their
    /// preconditions satisified.
    ///
    /// This is different from the restrictions on the method of the same name on `LongString`,
    /// where this is not always convertable.
    pub fn get_sized_buf(&self) -> NonNull<[u8]> {
        let ptr = self as *const Self as *const u8;
        unsafe {
            // SAFETY:
            // - note that we construct this from a `&self`, to comply with Stacked Borrows
            // - this operation is not safe, but we make assertions about the result, namely that
            //   it can be used as slice, so long as its lifetime is tied to a self parameter.
            //   Therefore, we must make sure that the slice is valid for slice::from_raw_parts
            //
            // - ptr add is within the object and len is at most sizeof(ShortString) - 1
            // - ptr.add(1).add(self.len()) is always guaranteed to be within the allocated object
            // - both are properly aligned because we're working with bytes
            let raw = ptr::slice_from_raw_parts(ptr.add(1), self.len());
            // SAFETY: ptr.add(1) cannot be null, as it is also a valid &[u8]
            NonNull::new_unchecked(raw as *mut [u8])
        }
    }

    /// Returns `buf[0..len]` as a `NonNull<[u8]>` with len `len`, you may cast the resulting slice
    /// to a `&'self str`  at any time, assuming all unsafe functions are called with their
    /// preconditions satisified.
    ///
    /// This is different from the restrictions on the method of the same name on `LongString`,
    /// where this is not always convertable.
    ///
    /// We need this method for this variant because the provenance of the returned slice is
    /// determined by the provenance of self. Therefore, if we used a `&self`, the region would be
    /// tagged with SharedReadOnly
    pub fn get_sized_buf_mut(&mut self) -> NonNull<[u8]> {
        let ptr = self as *mut Self as *mut u8;
        unsafe {
            // SAFETY:
            // - note that we construct this from a `&self`, to comply with Stacked Borrows
            // - this operation is not safe, but we make assertions about the result, namely that
            //   it can be used as slice, so long as its lifetime is tied to a self parameter.
            //   Therefore, we must make sure that the slice is valid for slice::from_raw_parts
            //
            // - ptr add is within the object and len is at most sizeof(ShortString) - 1
            // - ptr.add(1).add(self.len()) is always guaranteed to be within the allocated object
            // - both are properly aligned because we're working with bytes
            let raw = ptr::slice_from_raw_parts_mut(ptr.add(1), self.len());
            // SAFETY: ptr.add(1) cannot be null, as it is also a valid &[u8]
            NonNull::new_unchecked(raw as *mut [u8])
        }
    }

    /// interpret this string as a `&str`
    pub fn as_mut_str(&mut self) -> &mut str {
        // SAFETY: cast to `&'self mut [u8]` is always valid according to function description
        let buf = unsafe { &mut *self.get_sized_buf_mut().as_ptr() };
        // SAFETY: always valid utf-8, by definition
        unsafe { std::str::from_utf8_unchecked_mut(buf) }
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
pub struct LongString {
    /// # Safety
    /// - `0` is always a valid value
    /// - the last bit is always 0
    /// when shifted by >> 1:
    /// - `len <= capacity`
    /// - `buf[0..len]` is always a valid SharedReadWrite slice of valid u8, if the string is not
    ///    borrowed, otherwise the permissions become that of the borrow
    len: UnsafeField<usize, 0>,
    /// # Safety
    /// buf and capacity are linked, so we can only modify either if we update the entire struct
    /// simultaneously. As a result, we cannot implement Drop. The size of the allocated object
    /// starting at buf.data is always exactly capacity bytes long.
    buf: UnsafeField<RawBuf<u8>, 1>,
    /// # Safety
    /// buf and capacity are linked, so we can only modify either if we update the entire struct
    /// simultaneously. As a result, we cannot implement Drop. The size of the allocated object
    /// starting at buf.data is always exactly capacity bytes long.
    capacity: UnsafeField<usize, 2>,
}

impl LongString {
    /// Construct a new `LongString` with at least `capacity` as the `capacity`. Note that this
    /// will panic in the case of an impossible allocation (e.g. `capacity > isize::MAX`)
    pub fn with_capacity(capacity: usize) -> Self {
        let (buf, capacity) = RawBuf::new(capacity);

        unsafe {
            Self {
                // SAFETY: a value of `0` is always valid
                len: UnsafeField::new(0),
                // SAFETY: by definition of RawBuf::new, capacity and buf match, so both these
                // constructions are safe
                capacity: UnsafeField::new(capacity),
                buf: UnsafeField::new(buf),
            }
        }
    }

    /// interpret this as a `&str`
    pub fn as_str(&self) -> &str {
        // SAFETY: `LongString` always contains valid utf-8, buf[0..len] is always initialised
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        // SAFETY: conversion to `&'self mut [u8]` is valid, since we have not modified the buffer,
        // since acquiring the pointer (we immediately derefrenced)
        let buf = unsafe { &mut *self.get_sized_buf().as_ptr() };
        // SAFETY: always valid utf-8, by definition
        unsafe { std::str::from_utf8_unchecked_mut(buf) }
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

    pub fn get_non_null_slice(&self, index: usize, len: usize) -> Option<NonNull<[u8]>> {
        if index + len > self.capacity() {
            return None;
        }
        let Some(data) = self.get_non_null(index) else {
            return None;
        };

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
        // SAFETY: new has at least self.capacity() space, so it can allocate anything that
        // self holds
        unsafe {
            new.push_str_unchecked(self.as_str());
        }
        new
    }

    /// realloc to fit at least `remaining_capacity` more bytes
    pub fn realloc(&mut self, remaining_capacity: usize) {
        let new = self.clone_with_additional_capacity(cmp::max(
            remaining_capacity - self.remaining_capacity(),
            self.capacity() * 2,
        ));
        self.free();
        *self = new;
    }

    /// # Safety
    /// - `self.remaining_capacity()`` must be at least `s.len()`
    pub unsafe fn push_str_unchecked(&mut self, s: &str) {
        // SAFETY:
        // - src (s) is valid for reads of s.len() by slice definition
        // - dst is valid for writes of count s.len(), since remaining_capacity >= s.len()
        //   and self.next_ptr() points to a buffer of size remaining_capacity
        // - both dst and src are cast from aligned pointers
        // - the regions may not overlap, as `&mut` uniquely borrows the the buffer, thus
        //   `&str` must point somewhere else
        ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), self.next_ptr().as_ptr(), s.len());
        // SAFETY: just copied a valid str of s.len() into the section starting at len
        self.set_len(self.len() + s.len());
    }

    /// Push a `str` to this string, allocating if needed. Note that the current realloc schema
    /// might only allocate exactly enough extra space for `s`
    pub fn push_str(&mut self, s: &str) {
        if self.remaining_capacity() < s.len() {
            self.realloc(s.len());
        }

        // SAFETY: if remaining capacity is less than s.len(), we realloc to fit at least s.len()
        // therefore, the remaining capacity is at least s.len()
        unsafe { self.push_str_unchecked(s) }
    }

    /// Push a `char` to this string, allocating if needed. Like [`LongString::push_str`] this might
    /// only allocate enough extra space for `ch`, but that is very unlikely in this case.
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let utf8 = ch.encode_utf8(&mut buf);
        self.push_str(utf8);
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
                len: UnsafeField::new(0),
                // SAFETY: the buffer is dangling and the capacity is 0, which is a valid
                // state for LongString, these two fields have a linked invariant
                capacity: UnsafeField::new(0),
                buf: UnsafeField::new(
                    self.buf
                        .own()
                        // SAFETY: capacity is the exact size of the buffer
                        .dealloc(capacity)
                        .expect("should be the exact capacity"),
                ),
            }
        };
    }

    /// Construct a new `LongString` from a `length`, `buf` and `capacity`
    ///
    /// # Safety
    /// - invariants of `length`
    ///     - `0` is always a valid value
    ///     - `len <= capacity`
    ///     - `buf[0..len]` is always a valid SharedReadWrite slice of valid u8, if the string is not
    ///        borrowed, otherwise the permissions become that of the borrow
    /// - invariants of `buf` and `capacity`
    ///     - The size of the allocated object starting at buf is *exactly* `capacity` bytes long
    ///     - `buf` must be allocated with std::allocator::Global
    pub unsafe fn from_raw_parts(buf: NonNull<u8>, length: usize, capacity: usize) -> Self {
        Self {
            // SAFETY: invariants of `.len()` are passed to caller, so we must ensure the final bit
            // is `0`, which we do by shifting left 1.
            len: UnsafeField::new(length << 1),
            // SAFETY: passed to caller
            buf: UnsafeField::new(RawBuf { data: buf }),
            // SAFETY: passed to caller
            capacity: UnsafeField::new(capacity),
        }
    }

    pub fn from_str(s: &str) -> Self {
        let mut long = Self::with_capacity(s.len());
        // SAFETY: we allocate long with_capacity(s.len()). It is empty, therefore it must have
        // remaining_capacity == capacity == s.len()
        unsafe {
            long.push_str_unchecked(s);
        }
        long
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

pub enum TaggedSsoString64Mut<'a> {
    Short(&'a mut ShortString64),
    Long(&'a mut LongString),
}

pub enum TaggedSsoString64<'a> {
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

/// A wrapper around `str`, so that we can implement `ToOwned` where `ToOwned::Owned` is
/// `sso::String`
#[repr(transparent)]
pub struct SsoStr(str);

impl ToOwned for Str {
    type Owned = SsoString;

    fn to_owned(&self) -> Self::Owned {
        todo!()
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
    pub fn tagged(&self) -> TaggedSsoString64 {
        if self.is_short() {
            TaggedSsoString64::Short(unsafe { &self.short })
        } else {
            TaggedSsoString64::Long(unsafe { &self.long })
        }
    }

    /// Same as [`SsoString::tagged`], but returns allows mutation of the underlying values instead
    pub fn tagged_mut(&mut self) -> TaggedSsoString64Mut {
        if self.is_short() {
            TaggedSsoString64Mut::Short(unsafe { &mut self.short })
        } else {
            TaggedSsoString64Mut::Long(unsafe { &mut self.long })
        }
    }

    duck_impl! {
        /// Returns a slice of bytes of this string's contents
        pub fn as_bytes(&self) -> &[u8];
    }

    duck_impl! {
        pub fn as_mut_str(&mut self) -> &mut str;
    }

    never_impl!(pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8>);

    duck_impl! {
        pub fn as_str(&self) -> &str;
    }

    duck_impl! {
        pub fn capacity(&self) -> usize;
    }

    duck_impl! {
        pub fn clear(&mut self as duck) {
            // SAFETY: 0 is always a valid value for len on both variants
            unsafe { duck.set_len(0) }
        }
    }

    todo_impl! {
        pub fn drain<R>(&mut self, _range: R) -> Drain<'_>
        where
            R: RangeBounds<usize>,
    }

    todo_impl! {
        pub fn extend_from_within<R>(&mut self, _src: R)
        where
            R: RangeBounds<usize>,
    }

    /// Creates a new `SsoString::Long` from a length, capacity and pointer. This method only
    /// exists to match `std::string::String`'s method of the same signature and name. It will
    /// always create a long string, which is probably what you want if you are using this method.
    ///
    /// # Safety (from [`std::string::String`])
    ///
    /// This is highly unsafe, due to the numer of invariants that aren't checked:
    ///
    /// - The memory at buf needs to have been previously allocated by the same allocator the
    ///   standard library uses, with a required alignment of exactly 1.
    /// - `length` needs to be less than or equal to capacity.
    /// - `capacity` needs to be the correct value.
    /// - The first length bytes at buf need to be valid UTF-8.
    pub unsafe fn from_raw_parts(buf: *mut u8, length: usize, capacity: usize) -> Self {
        // SAFETY: safety contract passed to caller (buf must be nonnull)
        let ptr = NonNull::new_unchecked(buf);
        // SAFETY: safety contract passed to caller
        SsoString {
            long: ManuallyDrop::new(LongString::from_raw_parts(ptr, length, capacity)),
        }
    }

    todo_impl!(pub fn from_utf16(_v: &[u16]) -> Result<SsoString, FromUtf16Error>);

    todo_impl!(pub fn from_utf16_lossy(_v: &[u16]) -> SsoString);

    todo_impl!(pub fn from_utf8(_v: Vec<u8>) -> Result<String, FromUtf8Error>);

    todo_impl!(pub fn from_utf8_lossy(_v: &[u8]) -> Cow<'_, SsoStr>);

    todo_impl!(pub unsafe fn from_utf8_unchecked(_v: &[u8]) -> SsoString);

    todo_impl!(pub fn insert(&mut self, _idx: usize, _c: char));

    todo_impl!(pub fn insert_str(&mut self, _idx: usize, _s: &str));

    todo_impl!(pub fn into_boxed_str(self) -> Box<str>);

    todo_impl!(pub fn leak<'a>(self) -> &'a mut str);

    duck_impl! {
        pub fn len(&self) -> usize;
    }

    duck_impl! {
        pub fn push(&mut self, ch: char);
    }

    /// Push a str `s` onto the end of this string
    pub fn push_str(&mut self, s: &str) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Short(short) => {
                if s.len() <= short.remaining_capacity() {
                    // SAFETY: exact bounds check just completed
                    unsafe {
                        short.push_str_unchecked(s);
                    }
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

    todo_impl!(pub fn remove(&mut self, _idx: usize) -> char);

    todo_impl!(
        pub fn replace_range<R>(&mut self, _range: R, _replace_with: &str)
        where
            R: RangeBounds<usize>,
    );

    pub fn reserve(&mut self, additional: usize) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Short(short) => {
                let long = ManuallyDrop::new(short.into_long(additional));
                *self = SsoString { long };
            }
            TaggedSsoString64Mut::Long(long) => {
                long.realloc(additional);
            }
        }
    }

    duck_impl! {
        pub unsafe fn set_len(&mut self, len: usize);
    }

    duck_impl! {
        pub fn pop(&mut self as duck) -> Option<char> {
            let ch = duck.as_str().chars().rev().next()?;
            // SAFETY: will always still be valid utf8, as we are 'removing' a correctly sized utf8
            // byte sequence from the end of this string. For added assurance that this is safe,
            // this is basically exactly the same code as the std library impementation.
            unsafe {
                duck.set_len(duck.len() - ch.len_utf8());
            }
            Some(ch)
        }
    }

    /// This doesn't actually reserve exactly `additional` extra bytes, it might allocate a few
    /// extra, just because of the implementation of `Global`.
    pub fn reserve_exact(&mut self, additional: usize) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Short(short) => {
                let long = ManuallyDrop::new(short.into_long(additional));
                *self = SsoString { long };
            }
            TaggedSsoString64Mut::Long(old) => {
                let long = ManuallyDrop::new(old.clone_with_additional_capacity(additional));
                old.free();
                *self = SsoString { long };
            }
        }
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// # TODO
    ///
    /// This is a bad implementation of a simple function, it creates an entirely new string, it
    /// doesn't require `&mut`. The better version would be easier to implement with `as_mut_str`.
    ///
    /// I don't think I'm going to bother with this any time soon.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(char) -> bool,
    {
        macro_rules! duck_body {
            ($duck:ident, $f:ident) => {
                for ch in self.chars() {
                    if $f(ch) {
                        $duck.push(ch)
                    }
                }
            };
        }

        let mut result = SsoString::with_capacity(self.capacity());
        match result.tagged_mut() {
            TaggedSsoString64Mut::Long(long) => {
                duck_body!(long, f)
            }
            TaggedSsoString64Mut::Short(short) => {
                duck_body!(short, f)
            }
        }
        *self = result;
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= ShortString64::MAX_CAPACITY {
            Self {
                short: ManuallyDrop::new(ShortString64::new()),
            }
        } else {
            Self {
                long: ManuallyDrop::new(LongString::with_capacity(capacity)),
            }
        }
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        match self.tagged_mut() {
            TaggedSsoString64Mut::Long(old) => {
                let min_capacity = cmp::max(min_capacity, old.len());
                if min_capacity <= ShortString64::MAX_CAPACITY {
                    let mut short = ShortString64::new();
                    // SAFETY:
                    // 1. short is empty, therefore remaining_capacity == MAX_CAPACITY
                    // 2. old.len() <= min_capacity is true (cmp::max)
                    // 3. min_capacity <= MAX_CAPACITY is true (if statement)
                    // - therefore old.len() <= short.remaining_capacity() because of the following
                    //   derivation:
                    // -> simplifies... old.len() <= MAX_CAPACITY (1)
                    // -> given... old.len() <= min_capacity <= MAX_CAPACITY (2, 3)
                    // -> old.len() <= MAX_CAPACITY == true
                    unsafe {
                        short.push_str_unchecked(old.as_str());
                    }
                    old.free();
                    *self = SsoString {
                        short: ManuallyDrop::new(short),
                    };
                } else {
                    let mut long = LongString::with_capacity(min_capacity);
                    // SAFETY:
                    // - min_capacity >= old.len(), therefore old.len() <= min_capacity
                    // - we have allocated at least min_capacity for long, which is empty
                    // - therefore old.as_str() can be pushed
                    unsafe {
                        long.push_str_unchecked(old.as_str());
                    }
                    old.free();
                    *self = SsoString {
                        long: ManuallyDrop::new(long),
                    };
                }
            }
            TaggedSsoString64Mut::Short(..) => {
                // cannot shrink capacity any further
            }
        }
    }

    /// This is currently just an alias for `self.shrink_to(self.len())`, it doesn't avoid any
    /// branches just because it's always valid
    ///
    /// # TODO
    /// Implement this better
    pub fn shrink_to_fit(&mut self) {
        self.shrink_to(self.len());
    }

    todo_impl!(pub fn split_off(&mut self, _at: usize) -> String);

    todo_impl!(pub fn truncate(&mut self, _new_len: usize));

    todo_impl!(pub fn try_reserve(&mut self, _additional: usize) -> Result<(), TryReserveError>);

    todo_impl! {
        pub fn try_reserve_exact(&mut self, _additional: usize) -> Result<(), TryReserveError>;
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
            TaggedSsoString64::Short(short) => write!(f, "{:?}", short),
            TaggedSsoString64::Long(long) => write!(f, "{:?}", long),
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

impl Borrow<Str> for SsoString {
    fn borrow(&self) -> &Str {
        // SAFETY: transmute from &T to #[repr(transparent)] &Wrapper(T)
        unsafe { mem::transmute(self.as_str()) }
    }
}

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
pub type String = SsoString;

#[cfg(all(not(target_endian = "little"), not(target_pointer_width = "64")))]
pub type String = std::string::String;

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
pub type Str = SsoStr;

#[cfg(all(not(target_endian = "little"), not(target_pointer_width = "64")))]
pub type Str = str;
