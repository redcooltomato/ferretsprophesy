use crate::snek::{crossrender, genmap};

mod snek;


fn main() {
    let mut m = genmap(10, 10);
    crossrender(&mut m).expect("ssdfg");
}