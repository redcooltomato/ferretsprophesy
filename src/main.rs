use crate::snek::{crossrender, genmap};

mod snek;


fn main() {
    let mut m = genmap(15, 15);
    crossrender(&mut m).expect("game left with a error");
}