use crate::*;

/// function bodies from the alloc_layout_extra feature that I want to use on stable. Very much
/// robbery on my part, so credit to whoever wrote these originally
mod alloc_layout_extra {
    use super::*;

    pub const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
        let len = layout.size();

        // Rounded up value is:
        //   len_rounded_up = (len + align - 1) & !(align - 1);
        // and then we return the padding difference: `len_rounded_up - len`.
        //
        // We use modular arithmetic throughout:
        //
        // 1. align is guaranteed to be > 0, so align - 1 is always
        //    valid.
        //
        // 2. `len + align - 1` can overflow by at most `align - 1`,
        //    so the &-mask with `!(align - 1)` will ensure that in the
        //    case of overflow, `len_rounded_up` will itself be 0.
        //    Thus the returned padding, when added to `len`, yields 0,
        //    which trivially satisfies the alignment `align`.
        //
        // (Of course, attempts to allocate blocks of memory whose
        // size and padding overflow in the above manner should cause
        // the allocator to yield an error anyway.)

        let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
        len_rounded_up.wrapping_sub(len)
    }

    pub fn repeat(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
        // This cannot overflow. Quoting from the invariant of Layout:
        // > `size`, when rounded up to the nearest multiple of `align`,
        // > must not overflow isize (i.e., the rounded value must be
        // > less than or equal to `isize::MAX`)
        let padded_size = layout.size() + padding_needed_for(layout, layout.align());
        let alloc_size = padded_size.checked_mul(n)?;

        // The safe constructor is called here to enforce the isize size limit.
        let layout = Layout::from_size_align(alloc_size, layout.align()).ok()?;
        Some((layout, padded_size))
    }
}

/// guarantees layout is non-zero
pub fn new_slice_layout<T>(capacity: usize) -> (Layout, usize) {
    let (layout, len) =
        alloc_layout_extra::repeat(&Layout::new::<T>(), capacity).expect("capacity is valid");
    if layout.size() == 0 {
        panic!("cannot allocate ZST");
    }

    (layout, len)
}

/// To deallocate this, make sure you multiply by `mem::size_of<T>()`.
pub fn alloc_slice<T>(count: usize) -> NonNull<[T]> {
    let (layout, offset) = new_slice_layout::<T>(count);
    let (data, byte_count) = {
        #[cfg(feature = "nightly")]
        {
            let data = Global
                .allocate(layout)
                .unwrap_or_else(|_| panic!("allocation error"));
            (data.cast(), data.len())
        }
        #[cfg(not(feature = "nightly"))]
        {
            use std::alloc::alloc;
            // SAFETY: new_slice_layout guarantees that layout is non-zero
            let data = unsafe { alloc(layout) };
            let Some(data) = NonNull::new(data) else {
                panic!("allocation error")
            };
            (data, layout.size())
        }
    };
    // offset is the size of each allocation with padding
    let capacity = byte_count / offset;
    unsafe {
        // SAFETY: capacity * sizeof(T) is less than data.len(), so points to a valid slice
        let raw = ptr::slice_from_raw_parts_mut(data.as_ptr() as *mut _, capacity);
        // SAFETY: ptr is non-null, since `data.as_ptr()` is non-null
        NonNull::new_unchecked(raw)
    }
}

/// # Safety
/// must be a slice allocated by `unified_alloc::alloc_slice()`
pub unsafe fn dealloc_slice<T>(ptr: NonNull<[T]>) {
    let layout = new_slice_layout::<T>(ptr.len()).0;
    #[cfg(feature = "nightly")]
    {
        Global.deallocate(ptr.cast(), layout);
    }
    #[cfg(not(feature = "nightly"))]
    {
        use std::alloc::dealloc;
        // SAFETY: layout should be the same layout we made at the beginning, sicne it comes from
        // the same function
        unsafe {
            dealloc(ptr.as_ptr() as *mut _, layout);
        }
    }
}
