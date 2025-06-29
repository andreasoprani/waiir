use std::io::stdin;

mod lexer;
mod token;

fn main() {
    println!("Hello, this is the Monkey programming language!");
    println!("Feel free to type in commands");
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let mut line_lexer = lexer::Lexer::init(buf.as_str());
        loop {
            let tok = line_lexer.next_token();
            println!("{:?}", tok);
            if tok == token::Token::Eof {
                break;
            }
        }
    }
}
