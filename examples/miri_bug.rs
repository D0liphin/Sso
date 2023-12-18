use std::ptr::NonNull;

use sso::RawBuf;

fn main() {
    unsafe {
        let dangling_slice =
            std::slice::from_raw_parts::<'static, i32>(NonNull::dangling().as_ptr(), 0);
        println!("{dangling_slice:?}");
    }

    // Global.allocate() a 16+ sized buffer of i32
    let (buf, len) = RawBuf::<i32>::new(16);
    unsafe {
        buf.dealloc(len)
            .expect("len should be within an isize::MAX");
        let slice = std::slice::from_raw_parts(buf.as_ptr(), 0);
        println!("{slice:?}");
    };
}
