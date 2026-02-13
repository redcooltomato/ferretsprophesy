use crate::snek::{crossrender, genmap};

mod snek;


fn main() {
    let mut m = genmap(15, 20); // prob gonna hardcode the size
    crossrender(&mut m).expect("game left with a error");
}