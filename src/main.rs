/* use std::io; */
mod gaming;

fn main() {
    for i in 0..=49 {
        println!("{}", fib(i));
    }
}

fn fib(n: i32) -> i32 {
    match n {
        0 => 0,
        1 | 2 => 1,
        _ => {
            fib(n - 1) + fib(n - 2)
        },
    }
}