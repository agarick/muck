// note: much code taken from Michael Gattozzi's
//       https://github.com/mgattozzi/schemers

extern crate rustyline;
#[macro_use]
extern crate nom;

mod parser;

fn main() {
    let mut reader = rustyline::Editor::<()>::new();

    loop {
        match reader.readline("tack> ") {
            Ok(line) => {
                if line.trim() == "(exit)" { break; }
                else { println!("{}", line); }
            },
            Err(e) => {
                use rustyline::error::ReadlineError;
                match e {
                    ReadlineError::Eof |
                    ReadlineError::Interrupted => break,
                    _ => println!("readline error: {}", e),
                }
            },
        }
    }
}
