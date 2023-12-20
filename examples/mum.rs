use sso::String;
type StdString = std::string::String;

use std::{time, hint::black_box};

fn main() {
    let earlier = time::Instant::now();

    let mut v = Vec::with_capacity(1024);
    for _ in 0..100 {
        unsafe {
            v.set_len(0)
        }
        for _ in 0..1024 {
            v.push(StdString::from("Ildiko Iliffe"));
        }
        black_box(&v);
    }

    println!("old version took {:?} to run", time::Instant::now().duration_since(earlier));

    let mums_name = StdString::from("Ildiko Iliffe");
    let dads_name = String::from("Roger Iliffe");
    dbg!(dads_name);
    dbg!(mums_name);
}  