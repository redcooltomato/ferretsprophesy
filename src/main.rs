/* use std::io; */
mod stuff;

fn main() {
    let mut s = String::from("howdy!");
    s.push_str("\nim flowey, flowey the flower");
    println!("{s}");
}