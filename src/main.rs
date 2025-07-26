use std::io::stdin;

use waiir::Parser;

fn main() {
    println!("Hello, this is the Monkey programming language!");
    println!("Feel free to type in commands");
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let mut line_parser = Parser::init(buf.as_str());
        let program = line_parser.parse_program();
        for stmt in program.statements {
            println!("{:?}", stmt);
        }
    }
}
