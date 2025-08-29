use std::io::stdin;

use lexer::Lexer;
use token::Token;

fn main() {
    println!("Hello, this is the Monkey programming language!");
    println!("Feel free to type in commands");
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let mut line_lexer = Lexer::init(buf.as_str());
        loop {
            let tok = line_lexer.next_token();
            println!("{tok}");
            if tok == Token::Eof {
                break;
            }
        }
    }
}
