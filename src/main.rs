use std::io::stdin;
use std::rc::Rc;

use waiir::eval::{Environment, eval_with_env};

fn main() {
    println!("Hello, this is the Monkey programming language!");
    println!("Feel free to type in commands");
    let env = Rc::new(Environment::default());
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        println!("{}", eval_with_env(buf.as_str(), Rc::clone(&env)));
    }
}
