use std::io::stdin;

use waiir::eval_input;

fn main() {
    println!("Hello, this is the Monkey programming language!");
    println!("Feel free to type in commands");
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        println!("{}", eval_input(buf.as_str()));
    }
}
