/* use std::io; */
use crate::cross::{crossrender, genmap};

mod stuff;
mod cross;



fn main() {
    let mut m = genmap(10, 10);
    crossrender(&mut m).expect("ssdfg");
}